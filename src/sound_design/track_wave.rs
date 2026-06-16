use dasp::sample;
use egui::{
    Color32, Pos2, Rect, Ui,
    containers::{Frame, Window},
    emath, epaint,
    epaint::PathStroke,
     lerp, pos2, remap, vec2,
};


use eframe; 

use crate::{
    sound_design::{
        fft::FFTUtils,
    }
};

pub struct TrackWave {
    frame_index: usize, 
    freq_buf: Vec<f32>,
    // used to average with the last freq, to manage 0's and still have a smooth transition
    last_freq: f32, 
    sample_rate: f32, 
}

impl Iterator for TrackWave {
    
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.frame_index < self.freq_buf.len() {
            let freq = self.freq_buf[self.frame_index];
            self.frame_index += 1; 
            //self.last_freq = self.last_freq * 0.8 + freq * 0.2; 
            self.last_freq = freq;
            Some(self.last_freq) 
        } else {
            self.frame_index = 0;
            self.next()
        }
    }
}

impl TrackWave {
    
    pub fn new(file: String, sample_rate: f32) -> Self{
        let fft = FFTUtils::new(2048, sample_rate);
        Self {
            frame_index: 0,
            freq_buf: fft.render_track(file),
            sample_rate: sample_rate,
            last_freq: 0.0,
        }
    }
    
    pub fn freq_buf(&self) -> Vec<f32> {
        self.freq_buf.clone()
    }


} 
