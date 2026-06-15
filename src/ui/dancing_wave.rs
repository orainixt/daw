use dasp::sample;
use egui::{
    Color32, Pos2, Rect, Ui,
    containers::{Frame, Window},
    emath, epaint,
    epaint::PathStroke,
     lerp, pos2, remap, vec2,
};


use crate::{
    sound_design::{
        track_wave::TrackWave, 

    }
};


pub struct DancingWaves {

    ltracks: Vec<TrackWave>,
    points: Vec<Pos2>

}

impl DancingWaves {

    pub fn new(lfiles: Vec<String>, sample_rate: f32) -> Self {
        //this surely can be optimized
        let mut ltmp = vec![];

        for file in lfiles {
            let track = TrackWave::new(file, sample_rate); 
            ltmp.push(track);     
        }

        Self {
            ltracks: ltmp, 
            points: vec![],
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {

        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };


        Frame::canvas(ui.style()).show(ui, |ui| {


            let size = ui.available_width() * vec2(1.0, 0.35); 
            let (_id, rect) = ui.allocate_space(size); 

            let to_screen = emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), 
                rect
            ); 

            let mut shapes = vec![]; 

            for track in &mut self.ltracks {
                if let Some(freq) = track.next() {
                    let mode = remap(freq, 0.0..=20000.0, 1.0..=10.0) as f64; 
                    
                    let n = track.freq_buf().len();
                    
                    if n == 0 {continue;}

                    self.points.clear();

                    for i in 0..n {
                        let t = i as f64 / (n as f64) ; 
                        
                        let freq = track.freq_buf()[i]; 

                        let y = remap(freq, 0.0..=20000.0, 0.0..=1.0); 

                        self.points.push(to_screen * pos2(t as f32, y as f32));
                    }

                    let thickness = 2.0 / mode as f32; 
                    shapes.push(epaint::Shape::line(
                        self.points.clone(), 
                        PathStroke::new(thickness, color),
                    )); 

                }
            } 

            ui.painter().extend(shapes);

        }); 
    }
}
