use crate::{
    sound_design::{
        play::Play, 
        file_reader::FileReader,
    },
};

use std::{fs::File, sync::Arc};

use rustfft::{FftPlanner, num_complex::Complex, Fft};


pub struct FFTUtils {
    size: usize,
    fft:  Arc<dyn Fft<f32>>,
    sample_rate: f32,
}

impl FFTUtils {
    
    pub fn new(size: usize,  sample_rate: f32) -> Self {
        let mut planner = FftPlanner::new(); 
        let fft = planner.plan_fft_forward(size);

        Self {size,fft, sample_rate} 
    }

    fn render_samples(&self, samples: &[f32]) -> f32{
        
        // fft uses complex numbers, i.e. all sounds imaginary number are 0
        let mut buffer : Vec<Complex<f32>> = samples
            .iter()
            .map(|i| Complex {re: *i, im:0.0})
            .collect(); 
        
        self.fft.process(&mut buffer);
        
        // buffer is now Complex<f32> with im still at 0
        let magn : Vec<f32> = buffer
            .iter()
            .map(|i| i.norm())
            .collect(); 

        // now only the dominant is useful 
        // max_by takes 2 tuples (index, magnitude) and returns the maximum. 
        // index is the value of the frequency : index * sample_rate / frame_size 
        // only len/2 cuz the second part of the sprect is a mirror of the first

        magn[1..magn.len()/2] 
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _m)| i as f32 * self.sample_rate / self.size as f32) 
            .unwrap_or(0.0)
    }

    
    /// This only render the dominant frequency 
    pub fn render_track(&self, file: String) -> Vec<f32>{
        
        let mut freq_buf : Vec<f32> = vec![];
        
        let mut samples : Vec<f32> = vec![];

        
        // this is bad and should be handled properly
        let mut file_reader = FileReader::new(file).expect("no file exists"); 
        
        while let Some(sample) = file_reader.next() {
            samples.push(sample);

            if samples.len() == self.size {
                freq_buf.push(self.render_samples(&samples)); 
                samples = vec![]; 
            }
        }

        freq_buf
    }

}
