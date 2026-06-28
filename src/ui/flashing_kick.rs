use egui::{
    Ui, Rect, emath, epaint, Pos2, 
    containers::{Frame},
    vec2, 
};

pub struct FlashingKick {

    nb_points: usize,


}

impl FlashingKick {
    
    pub fn new() -> Self {
        todo!() ; 
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            
            ui.request_repaint(); 

            let size = ui.available_width() * vec2(1.0, 0.35); 
            let (_id, rect) = ui.allocate_space(size); 

            let to_screen = emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), 
                rect
            ); 

            let mut points : Vec<Pos2> = Vec::with_capacity(self.nb_points);

            for i in 0..self.nb_points {
                         
            }
            

        });
    }
}
