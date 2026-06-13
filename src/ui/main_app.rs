use eframe::{egui::Ui};

use crate::{
    ui::{
        app_build::AppBuild,
    }
};

pub struct MainApp {
    name: String, 
    value: f32, 
    label: String, 
    app_builder: AppBuild,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            name: String::from("DAW"), 
            value: 0.0, 
            label: String::from("uiii"), 
            app_builder: AppBuild::new(),
            
        }
    }
}

impl eframe::App for MainApp {

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
            // For inspiration and more examples, go to https://emilk.github.io/egui 

            egui::Panel::top("top_panel").show_inside(ui, |ui| {
                // The top panel is often a good place for a menu bar:
                
                self.app_builder.menu_bar(ui);
                
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                ui.heading("eframe template");
                
                self.app_builder.effects(ui);

            
            });
        }

}


