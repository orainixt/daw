#![allow(unused_imports)]

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

pub struct FrameData<'a>{
    freq_buf : &'a [f32],
    // used to average with the last freq, to manage 0's and still have a smooth transition
}


impl<'a> FrameData<'a>{
    
    pub fn new(freq_buf: &'a [f32]) -> Self{
        Self {
            freq_buf: freq_buf,
        }
    }

    pub fn get_slice(&self, track_index: usize, fft_size: usize) -> &[f32] {
        let start = track_index * fft_size / 2 ;
        let end = start + (fft_size / 2); 

        let slice = &self.freq_buf[start..end]; 


        slice
    }
    
    pub fn freq_buf(&self) -> &[f32] {
        &self.freq_buf
    }


} 
