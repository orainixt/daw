use symphonia::{
    core::{ audio::SampleBuffer,
            codecs::{Decoder, DecoderOptions}, 
            formats::{FormatOptions, FormatReader},
            meta::{MetadataOptions}, 
            probe::{Hint},
            io::MediaSourceStream,
    },
    default::{get_probe, get_codecs},
};

use std::{
    path::{PathBuf}, 
    fs::File, 
    sync::{
        {Arc},
        atomic::{AtomicU64, Ordering},
    }, 
};


/// Reads and decodes an audio file into a stream of f32 samples.
///
/// This structure uses Symphonia to decode audio data progressively.
pub struct FileReader{
    format: Box<dyn FormatReader>, 
    decoder: Box<dyn Decoder>,
    sample_buf: SampleBuffer<f32>,
    total_samples: Arc<Option<u64>>,
    nb_samples: Arc<AtomicU64>,
    index : u64, 
}

/// Iterator for FileReader to load samples in order one by one. 
/// Returns None when the file has been read.
impl Iterator for FileReader {
    type Item = f32;

    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    ///
    /// let mut reader = FileReader::new(String::from("song.mp3")).unwrap();
    ///
    /// // next() returns Some(f32) till there is samples
    /// // and None when the file has been explored
    /// while let Some(sample) = reader.next() {
    ///     println!("sample : {}", sample);
    /// }
    /// ```
    fn next(&mut self) -> Option<Self::Item> {

        let loop_idx = self.index as usize; 

        if loop_idx < self.sample_buf.samples().len(){
            self.nb_samples.fetch_add(1, Ordering::Relaxed) as usize;
            self.index += 1; 
            Some(self.sample_buf.samples()[loop_idx])
        } else {
            let packet = match self.format.next_packet() {
                Ok(pack) => pack, 
                _ => return None,
            };


            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d, 
                Err(_) => return None, 
            };
            
            self.sample_buf.copy_interleaved_ref(decoded);

            self.index = 0;

            self.next() 
        } 
    } 
}

/// Implementation of the FileReader structure.
/// reads an audio file and produces f32 audio samples.

impl FileReader {

    /// Creates a fileReader for the given audio file
    /// supports mp3, ogg and wav formats that are handled by Symphonia. 

    /// FileReader constructor
    /// 
    /// [@see](https://docs.rs/symphonia/latest/symphonia/default/fn.get_codecs.html#scraped-examples)
    ///
    /// # Arguments : 
    /// * `str_file`: Path to the audio file as a String.
    ///
    /// # Returns :
    /// * Ok(`fileReader::FileReader`) if the file was opened successfully
    /// * `Err` otherwise

    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    ///
    /// // We create a fileReader for an mp3 file
    /// let reader = FileReader::new(String::from("song.mp3"));
    /// assert!(reader.is_ok());
    /// ```
    pub fn new(str_file: String) -> Result<FileReader, Box<dyn std::error::Error>> {
        let mut file_path = PathBuf::new(); 
        file_path.push(str_file);

        let file = match File::open(&file_path) {
            Ok(f) => f, 
            Err(e) => { 
                println!("error while opening file : {}",e);
                return Err(Box::new(e));
            }
        };

        let mediastream = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();

        if let Some(ext) = file_path.extension() {
            hint.with_extension(ext.to_str().unwrap());
        }

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        
        let prob = get_probe().format(&hint,mediastream,&fmt_opts,&meta_opts);
        
        let mut format = prob.unwrap().format;


        let dec_opts: DecoderOptions = Default::default();

        let mut decoder = get_codecs().make(&format.default_track().unwrap().codec_params, &dec_opts)
            .expect("unsupported codec");
            
        let packet = format.next_packet()?;

        let decoded = decoder.decode(&packet).unwrap();
        let duration = decoded.capacity() as u64;
        let spec = *decoded.spec();      

        let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
        sample_buf.copy_interleaved_ref(decoded);

        // in order to get the correct number of samples, it needs to check how many channels there
        // is for the file. If it's mono it gonna be 1, stereo 2 etc. 

        let codec_params = &format.default_track().unwrap().codec_params; 
        let nb_channels = match codec_params.channels {
            Some(channels) => channels.count(), 
            None => 2, 
        }; 

        let real_nb_samples = codec_params.n_frames.map(|frames| frames*nb_channels as u64);

        Ok(FileReader{
            total_samples: Arc::new(real_nb_samples),
            format: format,
            decoder: decoder,
            sample_buf: sample_buf,
            nb_samples: Arc::new(AtomicU64::new(0)),
            index: 0,
        })              
    }
    
    pub fn get_nb_samples(&self) -> Arc<AtomicU64> {
       self.nb_samples.clone()
    }

    pub fn get_total_samples(&self) -> Arc<Option<u64>> {
        self.total_samples.clone()
    }
}
