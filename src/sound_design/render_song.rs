use std::fs::File;

use rustfft::{
    num_complex::Complex,
};

use crate::sound_design::{fft::FFTUtils, file_reader::{self, FileReader}};



pub struct RenderSong {
    
    nb_tracks: u32, 
    size: usize, 
    ltracks : Vec<String>,
    fft: FFTUtils,
    fps: f32, 
    max_amp : f32, 
}

impl RenderSong {

    pub fn new(
        nb_tracks: u32, 
        size: usize, 
        ltracks: Vec<String>,
        sample_rate: f32) -> Self { 

        Self {
            nb_tracks: nb_tracks,
            size: size, 
            ltracks: ltracks,
            fft: FFTUtils::new(size),
            fps: sample_rate / (size as f32),
            max_amp: 1.0, 
        }
    }

    pub fn render_song(&mut self) -> Vec<f32> {


        let first_file_reader = FileReader::new(self.ltracks[0].clone()).expect("can't open file");
        let samples = (*first_file_reader.get_total_samples()).expect("can't get samples");

        let total_frame = (samples + self.size as u64 - 1) / self.size as u64;

        let total_samples = self.nb_tracks as usize * (total_frame as usize + 1) * (self.size as usize / 2);

        let mut out_buf : Vec<f32> = vec![0.0 ; total_samples];
        let mut fft_buf: Vec<Complex<f32>> = vec![Complex{re: 0.0, im: 0.0} ; self.size as usize]; 
        
        for (idx, track) in self.ltracks.iter().enumerate() {

            let file_reader = FileReader::new(track.to_string()).expect("no file exists");


            self.fft.render_track(file_reader, idx, self.nb_tracks as usize, &mut fft_buf, &mut out_buf);
        }

        let max = out_buf.iter().cloned().fold(f32::NAN, f32::max); 
        
        if self.max_amp < max {
            self.max_amp = max;
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
