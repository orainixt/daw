#![allow(unused_imports)]

use crate::sound_design::{
        file_reader::FileReader, play::Play, track_wave
    };

use std::{
    fs::File, 
    sync::Arc
};

use rustfft::{FftPlanner, num_complex::Complex, Fft};


pub struct FFTUtils {
    size: usize,
    fft:  Arc<dyn Fft<f32>>,
}

impl FFTUtils {
    
    pub fn new(size: usize) -> Self {
        let mut planner = FftPlanner::new(); 
        let fft = planner.plan_fft_forward(size);

        Self {size,fft} 
    }

    fn render_samples(&self, samples: &[f32], fft_buf: &mut [Complex<f32>], out_buf: &mut [f32]){
        
        // fft uses complex numbers, i.e. all sounds imaginary number are 0
        for (sample, complex) in samples.iter().zip(fft_buf.iter_mut()) {
            complex.im = 0.0; 
            complex.re = *sample; 
        }
        
        self.fft.process(fft_buf);
        
        for (complex, out) in fft_buf[0..out_buf.len()].iter().zip(out_buf.iter_mut()) {
            *out = complex.norm();
        }


    }

    
    pub fn render_track(&self, mut file_reader: FileReader, track_index: usize, nb_tracks: usize, fft_buf: &mut [Complex<f32>], out_buf: &mut [f32]){


        let mut samples = Vec::with_capacity(self.size);
        let mut frame_cpt = 0; 



        while let Some(sample) = file_reader.next() {
            samples.push(sample);

            if samples.len() == self.size {

                let start = frame_cpt * nb_tracks * self.size / 2 + (track_index * self.size / 2); 
                let end = start + self.size / 2; 

                self.render_samples(&samples, fft_buf, &mut out_buf[start..end]);

                samples.clear();
                frame_cpt += 1; 
            }
        }

    }

}
