#![allow(unused_imports)]

use dasp::sample;
use egui::{
    Color32, Pos2, Rect, Ui,
    containers::{Frame, Window},
    emath, epaint,
    epaint::PathStroke,
     lerp, pos2, remap, vec2,
};


use crate::sound_design::{
        render_song::RenderSong, track_wave::FrameData 

    };


pub struct DancingWaves {
    
    magnitude: Vec<f32>,
    render: RenderSong,
    fps: f64,
    nb_tracks: usize,
    frame_index: usize,
}

impl DancingWaves {

    pub fn new(lfiles: Vec<String>, sample_rate: f32, size: usize) -> Self {
        //hardcoded values to test
        let nb_tracks = lfiles.len();
        let mut render = RenderSong::new(nb_tracks as u32, size, lfiles.clone(), sample_rate);
        
        Self {
            magnitude: render.render_song(),
            fps: (sample_rate as f64) / (size as f64),
            nb_tracks: nb_tracks,
            render: render,
            frame_index: 0,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {

        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };


        Frame::canvas(ui.style()).show(ui, |ui| {
            
            ui.request_repaint();
            let time = ui.input(|i| i.time);

            let size = ui.available_width() * vec2(1.0, 0.35); 
            let (_id, rect) = ui.allocate_space(size); 

            let to_screen = emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -1.0..=self.render.get_max_amp()), 
                rect
            ); 

            let curr_frame = (time * self.fps * 10.0) as usize;
            /**
            let curr_frame = self.frame_index; 
            self.frame_index += 1;
            */
            let size = self.render.get_size();

            
            let start = curr_frame * self.nb_tracks * size / 2; 
            let end = start + (self.nb_tracks * size / 2); 

            if self.magnitude.is_empty() || end > self.magnitude.len() { return; }

            let curr_frame_data = FrameData::new(&self.magnitude[start..end]);


            let mut shapes = vec![]; 

            let thickness = 2.0; 

            for i in 0..self.nb_tracks {

                let mut points : Vec<Pos2> = Vec::with_capacity(size / 2); 

                let track_slice = curr_frame_data.get_slice(i, size); 

                if track_slice.is_empty() {
                    continue;
                }

                for j in 0..size / 2{
                    // start drawing 
                    let t = j as f32 / (size / 2) as f32 ;
                    let amp = track_slice[j];

                    points.push(to_screen * pos2(t, amp)); 
                }
                
                shapes.push(epaint::Shape::line(
                    points, 
                    PathStroke::new(thickness, color),
                )); 

            }
           ui.painter().extend(shapes);

        }); 
    }
}
