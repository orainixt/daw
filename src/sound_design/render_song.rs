use std::{
    fs,
    fs::File,
    io::{Write},
    slice,
    convert::TryInto,
};

use format_bytes::format_bytes;

use rustfft::{num_complex::Complex, num_traits::ToBytes};

use crate::sound_design::{fft::FFTUtils, file_reader::FileReader};

use bytemuck;

use log::{info};

// might be over-engineered
trait EndianRead{
    fn read_be(input: &mut &[u8]) -> Self; 
}

macro_rules! impl_EndianRead_for_nums (( $($num:ident),*) => {
    $(
        impl EndianRead for $num {
            fn read_be(input: &mut &[u8]) -> Self {
                let (bytes, rest) = input.split_at(std::mem::size_of::<Self>()); 
                *input = rest ; 
                Self::from_be_bytes(bytes.try_into().unwrap())
            }
        }
    )*
});


impl_EndianRead_for_nums!(f32, u64, usize);


pub struct DancingWaveUtils {
    
    nb_tracks: u32, 
    size: usize, 
    ltracks : Vec<String>,
    fft: FFTUtils,
    fps: f32, 
    max_amp : f32,
    name: String
}

impl DancingWaveUtils {

    pub fn new(
        nb_tracks: u32, 
        size: usize, 
        ltracks: Vec<String>,
        sample_rate: f32,
        name: String) -> Self { 


        Self {
            nb_tracks: nb_tracks,
            size: size, 
            ltracks: ltracks,
            fft: FFTUtils::new(size),
            fps: sample_rate / (size as f32),
            max_amp: 1.0, 
            name: name,
        }
    }
    
    fn f32_to_u8<'a>(&self, lfoats : &'a [f32]) -> &[u8] {
        unsafe {
            slice::from_raw_parts(lfoats.as_ptr() as *const _ , lfoats.len() * 4)
        }
    }

    pub fn u8_to_f32<'a>(&self, lunsigneds: &'a[u8]) -> &[f32]{
        unsafe {
            slice::from_raw_parts(lunsigneds.as_ptr() as *const _, lunsigneds.len() / 4)
        }
    }


    pub fn render_song(&mut self) {
        
        let mut out_file = File::create(&self.name).expect("File can't be opened nor created") ;
        
        let first_file_reader = FileReader::new(self.ltracks[0].clone()).expect("can't open file");
        let samples = (*first_file_reader.get_total_samples()).expect("can't get samples");

        let total_frame = (samples + self.size as u64 - 1) / self.size as u64;

        let total_samples = self.nb_tracks as usize * (total_frame as usize + 1) * (self.size as usize / 2);
        
        let b_total_samples = &total_samples.to_le_bytes(); 


        info!("{:#?}", b_total_samples);


        let _res = out_file.write_all(&total_samples.to_ne_bytes());

        let mut out_buf : Vec<f32> = vec![0.0 ; total_samples];
        let mut fft_buf: Vec<Complex<f32>> = vec![Complex{re: 0.0, im: 0.0} ; self.size as usize]; 
        
        // need to use rayon to do that when multiple tracks

        for (idx, track) in self.ltracks.iter().enumerate() {

            let file_reader = FileReader::new(track.to_string()).expect("no file exists");

            self.fft.render_track(file_reader, idx, self.nb_tracks as usize, &mut fft_buf, &mut out_buf);

        }
        let max = out_buf.iter().cloned().fold(f32::NAN, f32::max); 
        
        if self.max_amp < max {
            self.max_amp = max;
        }
        
        let buf : &[u8] = self.f32_to_u8(&out_buf);
        
        let _ = out_file.write_all(buf); 
    
    }

    pub fn parse_song(&self) -> Vec<f32>{
        
        //info!("before file/buf content");
        let file_content = fs::read(&self.name).expect("couldn't read file");
        let mut byte_content = &file_content[..]; 

        //info!("before total_samples");
        // std::mem::size_of<T>() is evaluated at compile time
        let total_samples = <usize as EndianRead>::read_be(&mut byte_content) as usize; 
        
        //info!("total_samples");

        info!("{}", total_samples); 
        info!("beofre out_buf");
        let out_buf : Vec<f32> = vec![0.0 ; total_samples];

        info!("before to_read");
        let to_read = std::mem::size_of::<f32>() * total_samples;

        info!("the for loop is panicking");
        
        //let total_frame = <f32 as EndianRead>::read_be(&mut byte_content) ; ; 

        for _ in 0..to_read {
            let sample = <f32 as EndianRead>::read_be(&mut byte_content) ; 
            info!("{}", sample);
        }   

        out_buf

    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_max_amp(&self) -> f32 {
        self.max_amp
    }
}
