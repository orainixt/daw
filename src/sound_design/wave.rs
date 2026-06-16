#![allow(unused_imports)]


use std::f32::consts::PI;


const BUF_SIZE: usize = 256; 
const SAMPLE_RATE: i32= 44100; 

#[derive(Debug)]
pub struct Wave {
    amp: f32, // amplitude
    ang_freq: f32, 
    phase: f32,

    sample_buf: [f32; 256],
    index: usize, 
}

impl Iterator for Wave {
    type Item = f32; 
    fn next(&mut self) -> Option<Self::Item>{

        if self.index < self.sample_buf.len() {
            let sample_v = self.sample_buf[self.index];
            self.index += 1;
            Some(sample_v)
        } else {
            self.fill256(); 
            self.index = 1; 
            let sample_v = self.sample_buf[0]; 
            Some(sample_v)
        }
    }
}

// en train de résoudre le pb de fill256 
// se renseigner sur la phase pour les temps grands 

impl Wave {
    pub fn new(amp: f32, ord_freq: f32, phase: f32)-> Self {
        Self {
            amp: amp, 
            ang_freq: 2.0 * PI * ord_freq, 
            phase: phase,
            sample_buf: [0.0;256], 
            index: 0,
        }
    }

    fn fill256(&mut self){
        let phase_incr = self.ang_freq / (SAMPLE_RATE as f32); 
        for i in 0..BUF_SIZE {
            self.phase += phase_incr ; 
            if self.phase > 2.0 * PI {
                self.phase -= 2.0 * PI; 
            }
            let y = self.amp * self.phase.sin() ;
            self.sample_buf[i] = y; 
        }
    }
}
