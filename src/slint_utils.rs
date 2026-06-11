use slint::{
    SharedPixelBuffer, 
    Rgba8Pixel, 

};

use tiny_skia::{
    Paint, 
    PixmapMut, 
    PathBuilder,
    Stroke,
    LineCap,
    Transform,

};

pub struct SlintUtils {}

impl SlintUtils {
    
    /// Arguments 
    /// * f_width f32 because u32 is more logic but would round before needed.
    pub fn gen_checkered_bg(n: u32, m: u32, f_width: f32, f_height: f32, wave: String) -> SharedPixelBuffer::<Rgba8Pixel> {
        
        

        let u_width = f_width as u32; 
        let u_height = f_height as u32; 

        let cell_width = f_width / (n as f32) ; 
        let cell_height = f_height / (m as f32);
        
        let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(u_width, u_height);

        let mut pixmap = PixmapMut::from_bytes(
            pixel_buffer.make_mut_bytes(), u_width , u_height
        ).unwrap();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);

        let mut paint1 = Paint::default();
        paint1.set_color_rgba8(0, 0, 0, 200);
        paint1.anti_alias = false;

        let mut paint2 = Paint::default();
        paint2.set_color_rgba8(0, 125, 0, 200);
        paint2.anti_alias = false;

        let path1 = {
            let mut pb = PathBuilder::new(); 
            
            let mut pb_width = cell_width; 
            while pb_width < f_width{
                pb.move_to(pb_width, 0.0); 
                pb.line_to(pb_width, f_height);
                pb_width += cell_width; 
            }

            let mut pb_height = cell_height; 
            while pb_height < f_height{
                pb.move_to(0.0 , pb_height); 
                pb.line_to(f_width, pb_height);
                pb_height += cell_height;
            }

            pb.finish().unwrap()
        };

        let path2 = {
            let mut pb = PathBuilder::new();

            let mut x: u32 = 0; 
            let mut y: f32 = 0.0;

            let amplitude : f32 = 20.0; 
            let frequency : f32 = 20.0;

            while x < u_width {
                y = (f_height / 2.0) + amplitude * f32::sin(x as f32 / frequency);
                pb.line_to(x as f32, y);
                x += 1; 
            }

            pb.finish().unwrap() 
        };

        let mut stroke1 = Stroke::default();
        stroke1.width = 1.0;
        stroke1.line_cap = LineCap::Round;

        let mut stroke2 = Stroke::default(); 
        stroke2.width = 3.0;
        stroke2.line_cap = LineCap::Round;

        pixmap.stroke_path(&path1, &paint1, &stroke1, Transform::identity(), None);
        pixmap.stroke_path(&path2, &paint2, &stroke2, Transform::identity(), None);

        pixel_buffer

    }
}
