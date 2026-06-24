#![allow(unused_imports)]

use egui::{
    Color32, Pos2, Rect, Ui,
    containers::{Frame, Window},
    emath, epaint,
    epaint::PathStroke,
    lerp, pos2, remap, vec2,
};


use log::{info};

use std::{
    fs::{File}  
};

use crate::sound_design::{
        render_song::{self, DancingWaveUtils}, track_wave::FrameData 

    };



/** 
* In the end it's not useful (for now)
trait ReadAtPosition {
    fn read_at_position(&mut self, pos: u64, buffer: &mut [u8]) -> std::io::Result<()>;
}

impl ReadAtPosition for File {
    fn read_at_position(&mut self, pos: u64, buffer: &mut [u8]) -> std::io::Result<()> {
        self.seek(SeekFrom::Start(pos))?;
        self.read_exact(buffer)
    }
}
*/
pub struct DancingWaves {
    
    magnitude: Vec<f32>,
    render: DancingWaveUtils,
    fps: f64,
    nb_tracks: usize,
    frame_index: usize,
}

impl DancingWaves {

    pub fn new(lfiles: Vec<String>, sample_rate: f32, size: usize, name: String) -> Self {
        //hardcoded values to test
        let nb_tracks = lfiles.len();
        let mut render = DancingWaveUtils::new(nb_tracks as u32, size, lfiles.clone(), sample_rate, name.clone());
        
        info!("before render song");
        // This is a bit silly to do
        // (render song in a file then read directly this file) 

        info!("render_song ok \nbefore parse_song");

        
        Self {
            magnitude: render.open_song(),
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

            let curr_frame = (time * self.fps) as usize;
            /**
            let curr_frame = self.frame_index; 
            self.frame_index += 1;
            */
            let size = self.render.get_size();

            
            let start = curr_frame * self.nb_tracks * size / 2; 
            let end = start + (self.nb_tracks * size / 2); 

            if end > self.magnitude.len() { return; }


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
                    PathStroke::new_uv(thickness, move |rect, p| {
                        let t = remap(p.x, rect.x_range(), -1.0..=1.0).abs();

                        let center_color = Color32::from_hex("#5BCEFA").expect("color cant be created"); 
                        let outer_color = Color32::from_hex("#F5A9B8").expect("color can't be created");

                        Color32::from_rgb(
                            lerp(center_color.r() as f32..=outer_color.r() as f32, t) as u8,
                            lerp(center_color.g() as f32..=outer_color.g() as f32, t) as u8,
                            lerp(center_color.b() as f32..=outer_color.b() as f32, t) as u8,
                        )
                    })
                )); 

            }
           ui.painter().extend(shapes);

        }); 
    }
}
