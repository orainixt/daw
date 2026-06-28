pub struct DisplayClip {}

impl DisplayClip {

    pub fn ui(&mut self, ui: &mut Ui) {
        

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.request_repaint(); 
            
            

        });
    }
}
