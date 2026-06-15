use eframe::{egui::Ui};
use egui::{
    Color32, Key::D, Rect, Vec2, containers::Frame, emath::{self, Pos2}, epaint::{self, PathStroke}, pos2, widgets 
};
use crate::{
    ui::{
        app_build::AppBuild,
        consts,
        dancing_wave::DancingWaves,
    }
};


pub struct MainApp {
    name: String, 
    value: f32, 
    label: String, 
    app_builder: AppBuild,
    dancing_wave: DancingWaves,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            name: String::from("DAW"), 
            value: 0.0, 
            label: String::from("uiii"), 
            app_builder: AppBuild::new(),
            dancing_wave: DancingWaves::new(vec![String::new()], 44100.0),
            
        }
    }
}

impl eframe::App for MainApp {

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
            // For inspiration and more examples, go to https://emilk.github.io/egui 
            
            ui.set_width(consts::F_GLOBAL_WIDTH);
            ui.set_height(consts::F_GLOBAL_HEIGHT);

            egui::Panel::top("top_panel").show_inside(ui, |ui| {
                // The top panel is often a good place for a menu bar:
                
                self.app_builder.menu_bar(ui);
                
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                ui.heading("eframe template");
                
               self.dancing_wave.ui(ui); 
            
            });
        }

}


