use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    volume::Volume,
};

use std::{
    sync::{Arc, Mutex,},
};


/// Structure for Play
/// Play audio samples thanks to the sound card
/// Uses cpal traits to send the audio samples to the output devices
/// The volume is shared via `Arc<Mutex>` to allow real-time modification.
pub struct Play{
    file_audio: Arc<Mutex<Volume>>,
    stream: Option<cpal::Stream>, 
}



// initiate the host
// choose the devise (default device)
// and create a stream
// finalement j'ai decidé d'implementer le volume en type Arc<Mutex> c une valeur qui sera 
// partagée entre le play et le cpal donc le mieux etait de l'utiliser comme un arc mutex
impl Play{
    /// Creates a new Play with the given Volume
    ///
    /// # Arguments
    /// * `file_audio` it's a volume instance that contains the file audio and the volume value
    
    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    /// use Audio_Player::volume::Volume;
    /// use Audio_Player::play::Play;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let file = FileReader::new(String::from("song.mp3")).unwrap();
    /// let vol = Arc::new(Mutex::new(0.5f32));
    /// let volume = Volume::new(file, Arc::clone(&vol));
    ///
    /// // we create the Player with the volume(that contains the file also)
    /// let player = Play::new(volume);
    /// ```
    pub fn new(file_audio: Volume) -> Play { 
        
        Play{
            file_audio: Arc::new(Mutex::new(file_audio)),
            stream: None,
        }
    }

    /// returns a handle to the volume like Arc_Mutex for real time control for the volume 
    /// So we can modify the volume while the audio is being played
    ///  # returns
    /// * `Arc<Mutex<f32>>` a shared reference for the volume
        
    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    /// use Audio_Player::volume::Volume;
    /// use Audio_Player::play::Play;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let file = FileReader::new(String::from("song.mp3")).unwrap();
    /// let vol = Arc::new(Mutex::new(0.5f32));
    /// let volume = Volume::new(file, Arc::clone(&vol));
    /// let player = Play::new(volume);
    /// let handle = player.get_volume_handle();
    /// let mut v = handle.lock().unwrap();
    /// *v = 0.8;
    /// ```
    pub fn get_volume_handle(&self)-> Arc<AtomicF32>{
        self.file_audio.lock().unwrap().get_volume()
    }

    /// This function allows to play the samples through the device output 
    /// This function is called in a separate thread to avoid blocking the main thread.
    
    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    /// use Audio_Player::volume::Volume;
    /// use Audio_Player::play::Play;
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let file = FileReader::new(String::from("song.mp3")).unwrap();
    /// let vol = Arc::new(Mutex::new(0.5f32));
    /// let volume = Volume::new(file, Arc::clone(&vol));
    /// let mut player = Play::new(volume);
    /// thread::spawn(move || {
    ///     player.play_samples();
    /// });
    /// ```
    pub fn play_samples(&mut self) -> (){
        //sur linux le systeme audio est ALSA, cpal le choisit par default
        let host = cpal::default_host();
        
        /* debug
        for d in cpal::default_host().devices().unwrap(){
            println!("Found device {}", d.description().unwrap());
        }
        */   
        
        // If yout have a "NoDeviceDetected" or something related to the default device 
        // @see https://github.com/Uberi/speech_recognition/issues/526
        //
        // on récupére la carte son par défaut de notre os
        let device = host.default_output_device().unwrap();
        // on récupére les configurations des formats acceptés       
         

        // on choisit la bonne configuration
        //self.samples = self.file_audio.extract_samples().expect("Extract failed").into();   

              
        let supported_config = device.default_output_config().expect("no default output config ?"); 
        let def_sample_rate = supported_config.sample_rate(); 
        let def_channels = supported_config.channels(); 

        let optimum_buf_size = 1024; 

        let buffer_size = match supported_config.buffer_size() {

            cpal::SupportedBufferSize::Range{min, max} => {
                // if the optimum_buf_size is greater than max it takes tha max, if it's lower than
                // min it chooses the min. Otherwise it returns the optimum_buf_size
                let buf_size = optimum_buf_size.clamp(*min, *max);
                cpal::BufferSize::Fixed(buf_size)
            } 

            _ => cpal::BufferSize::Default 
        }; 
        
        let config = cpal::StreamConfig {
            channels: def_channels, 
            sample_rate: def_sample_rate, 
            buffer_size: buffer_size,
        };

        let file_audio_cpy = Arc::clone(&self.file_audio);

        self.stream = Some(device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                let mut locked_file_audio_cpy = file_audio_cpy.lock().unwrap();
                for sample in data.iter_mut() {

                    if let Some(sample_from_volume) = locked_file_audio_cpy.next() {
                        *sample = sample_from_volume; 
                    } else {
                        *sample = 0.0; 
                    }
                }
            },
            move |err| {
                println!("Erreur (play) : {}", err);
            },
            None
        ).unwrap());

        self.stream.as_ref().expect("stream cutted").play().unwrap();
        
    }
    
    /// This function is used to pause/stop the playback 
    ///
    /// Setting `self.stream` to None is allowed here because : 
    ///     - The iterator inside `FileReader` keeps track of the index / sample buffer
    ///     - The pointing reference (Arc) of the `file_audio` inside the closure of play_samples
    ///     allows the main reference to live, even when the stream from the closure is deleted.
    /// 
    /// `pause_samples` will then recreates the stream from the reference of file_audio (not the Arc
    /// one) and starts the iterator again, meaning that the recreated stream is exactyly where it
    /// was when it was destroyed. 
    ///
    pub fn pause_samples(&mut self) {
        self.stream = None;
    }

    pub fn get_stream(&self) -> &Option<cpal::Stream>{
        &self.stream
    }
}
