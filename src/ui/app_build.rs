#![allow(unused_imports)]

use eframe::egui; 

use egui::{
    Rect, containers::Frame, emath::{self, Pos2}, epaint::{self, PathStroke}, pos2, widgets, Color32, Vec2, 
};
use env_logger::fmt::style::Color;

use crate::{
    ui::{
        consts,
    }
};

pub struct AppBuild {
}

impl AppBuild {

    pub fn new() -> Self {
        Self {
        }
    } 

    pub fn menu_bar(&self, ui:&mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            // NOTE: no File->Quit on web pages!
            let is_web = cfg!(target_arch = "wasm32");
            if !is_web {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
            }

            egui::widgets::global_theme_preference_buttons(ui);
        });
    }

    fn render_oscillator(&self, ui: &mut egui::Ui) {

        Frame::canvas(ui.style()).show(ui, |ui| {

            ui.set_width(consts::F_OSCILLATOR_W);
            ui.set_height(consts::F_OSCILLATOR_H);

            ui.request_repaint();
            

            let time = ui.input(|i| i.time);

            //let desired_size = ui.available_size(); 
 
            let mut shapes = vec![];

            let n = 120; 
            let amp : f64 = 30.0; 
            let freq : f64 = 3.0;            
            let size = Vec2::new(consts::F_OSCILLATOR_W, consts::F_OSCILLATOR_H);  

            let (_id, rect) = ui.allocate_space(size);

            ui.set_clip_rect(rect);

            let to_screen = emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -amp as f32..=amp as f32), 
                rect
            );

 
            let points : Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f64 / n as f64;
                    let y = amp * (t * std::f64::consts::TAU * freq).sin();
                    to_screen * pos2(t as f32, y as f32) 
                })
                .collect() ; 

            
            let thickness = 10.0 ; 

            shapes.push(epaint::Shape::line(
                points, 
                PathStroke::new(thickness, Color32::GREEN), 
            )); 

            ui.painter().extend(shapes);
        });
    }
    
    
    fn wave_select(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("Select a wave");
            ui.set_width(consts::F_WAVE_W);
            ui.set_height(consts::F_WAVE_H);

            self.render_oscillator(ui);
        });
    }

    pub fn effects(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_width(consts::F_EFFECTS_W);
            ui.set_height(consts::F_EFFECTS_H);
            self.wave_select(ui);
        }); 
    }

}
