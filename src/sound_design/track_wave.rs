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
    sample_rate: f32, 
}

impl Iterator for TrackWave {
    
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.frame_index < self.freq_buf.len() {
            self.frame_index += 1; 
            Some(self.freq_buf[self.frame_index]) 
        } else {
            self.frame_index = 0;
            self.next()
        }
    }
}

impl TrackWave {
    
    pub fn new(file: String, sample_rate: f32) -> Self{
        let fft = FFTUtils::new(1024, sample_rate);
        Self {
            frame_index: 0,
            freq_buf: fft.render_track(file),
            sample_rate: sample_rate,
        }
    }
    
    pub fn freq_buf(&self) -> Vec<f32> {
        self.freq_buf.clone()
    }


} 
