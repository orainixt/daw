#![allow(unused)]

use crate::{AppWindow, 
    fileReader::FileReader, 
    play::Play, 
    volume::{Volume, SourceType},
    wave::{Wave} 
};  
use log::debug;


use slint::{
    StandardListViewItem, 
    SharedPixelBuffer, 
    Rgba8Pixel, 
    Image
}; 

use tiny_skia::{
    PathBuilder, 
    Pixmap, 
    Paint,
    FillRule,
    Transform, 
    PixmapMut,
    Stroke,
    StrokeDash, 
    LineCap,

}; 
use std::sync::Mutex;

use std::{
    sync::{
        mpsc,
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    io, 
    fs::{self}, 
    path::Path,
    rc::Rc,
};

/// Enum of the differents commands send by the UI callbacks (@see
/// UICommandsSender::setup_callbacks())
#[derive(Debug)]
pub enum UICommands{
    SwitchPlayMode(),
    ChangeVolume(f32), 
    ClickedInsidePopup(i32),
    LoadFile(), 
    FetchDirectory(),
    ImportFile(),
    UpdateProgressBar(),
    StopSamples(),
    SendSinWave(),
    SendSinGraph(f32, f32),
    Quit(),

}

/// Structure of the Sender used to wrap aroung the producer from the mpsc communication channel
/// (@see main)
///
/// # Arguments : 
/// * `ui` : Weak[^note] copy of the `slint::Weak<AppWindow>`
/// * `tx` : Clone of the `mpsc::Sender`
///
/// [^note]: A Weak reference does not know if the Application is still running.
/// It implements Clone traits, allowing us to give a reference to this structure
/// When something needs to be done or change on the UI, this reference is upgraded and the instruction is sent
/// through a buffered cannal (@see `slint::upgrade_in_event_loop()` & `slint::invoke_from_event_loop()`, but I'll talk more about it in UICommandsReceiver::match_command()) 
pub struct UICommandsSender{
    ui: slint::Weak<AppWindow>, 
    tx: mpsc::Sender<UICommands>,
} 

/// Implementation of the struct UICommandsSender
impl UICommandsSender{
    
    /// UICommandsReceiver constructor 
    ///
    /// # Arguments : 
    ///
    /// * `ui` : Weak[^note] copy of the `slint::Weak<AppWindow>`
    /// * `tx` : Clone of the `mpsc::Sender`
    ///
    /// # Returns :  
    /// * `UICommandsSender`
    ///
    /// # Exemple : 
    /// ```ignore
    /// # use std::sync::mpsc; 
    /// # use Audio_Player::{AppWindow,slint_logic};
    /// // tx is a mpsc::Sender, which sends messages to mspc::Receiver
    /// let (tx, _) = mpsc::channel::<slint_logic::UICommands>();
    ///
    /// let ui = AppWindow::new().unwrap();
    /// let ui_weak = ui.as_weak();
    /// let mut sender = slint_logic::UICommandsSender::new(ui_weak.clone(), tx.clone());
    /// ```
    ///
    pub fn new(ui : slint::Weak<AppWindow>, tx: mpsc::Sender<UICommands>) -> Self{

        Self {
            ui: ui,
            tx: tx, 
        }
    }
    
    /// Function used to setup all the callbacks from the [App](./../ui/app-window.slint)
    ///
    /// The `mpsc` library, and especially `mspc::channel()` function, defines the `tx` parameter, which is a `mpsc::Sender`. 
    /// The main purpose of this library is that this sender can be clone safelly. Each callback
    /// closure moves a copy of the `mpsc::Sender`
    ///
    /// The `mpsc::Receiver` object is receiving senders message inside a thread.
    ///
    /// In my opinion this is better than spawing a thread for each callback. 
    /// Each call only needs to send an `UICommands` field and furthermore the `mpsc` library is
    /// buffered, so calls can stack and be executed sequentially.
    ///
    /// # Exemple : 
    /// ```ignore
    /// # use Audio_Player::{AppWindow, slint_logic};
    /// let ui = AppWindow::new().unwrap();
    /// let ui_weak = ui.as_weak();
    /// let mut sender = slint_logic::UICommandsSender::new(ui_weak.clone(), tx.clone());
    /// sender.setup_callbacks();
    /// ```
    ///
    pub fn setup_callbacks(&mut self){
                
        /// Weak reference allows also cloning, which the "main" object won't
        let _ui_cpy = self.ui.clone();

        if let Some(ui) = self.ui.upgrade() {

            let tx_cpy = self.tx.clone();
            ui.on_request_fetch_directory(
                move ||{
                    tx_cpy.send(UICommands::FetchDirectory());
            });

            let tx_cpy = self.tx.clone();
            ui.on_request_user_selected_file(
                /// the callbacks parameters defined in the Slint file can be caught like that.
                move |index| {
                    tx_cpy.send(UICommands::ClickedInsidePopup(index));
            });

            let tx_cpy = self.tx.clone();
            ui.on_request_load_file(
                move || {
                    tx_cpy.send(UICommands::LoadFile());
            });

            let tx_cpy = self.tx.clone();
            ui.on_request_play_pause(
                move ||{
                    tx_cpy.send(UICommands::SwitchPlayMode());
            });

            let tx_cpy = self.tx.clone();
            ui.on_request_import_file(
                move ||{
                    tx_cpy.send(UICommands::ImportFile());
                }
            );

            let tx_cpy = self.tx.clone();
            ui.on_request_stop(
                move ||{
                    tx_cpy.send(UICommands::StopSamples());
                }
            );

            let tx_cpy = self.tx.clone();
            ui.on_request_progress_bar(
                move ||{
                    tx_cpy.send(UICommands::UpdateProgressBar());
                }
            );

            let tx_cpy = self.tx.clone();
            ui.on_request_change_volume(move |value| {
                tx_cpy.send(UICommands::ChangeVolume(value));
            });

            let tx_cpy = self.tx.clone();
            ui.on_request_add_sinwave(move || {
                tx_cpy.send(UICommands::SendSinWave());
            });

            let tx_cpy = self.tx.clone();
            ui.on_request_singraph(move |height, width| {
                println!("send order");
                tx_cpy.send(UICommands::SendSinGraph(height, width)); 
            });
            
            let tx_cpy = self.tx.clone();
            ui.on_request_close_app(move ||{
                tx_cpy.send(UICommands::Quit());
            });
        }
    }


}


/// Structure used to save user data once the application is up 
///
/// # Arguments : 
///
/// * `files` : `Vec<String>` used to store the files of the ./data_samples folder. 
/// * `ui` : `slint::Weak<AppWindow>` reference, which is a Weak[^note] version of the App
/// reference, which can be clone.
/// * `selected_file` : `String`, empty when started, when the `UICommands::ClickedInsidePopup()`
/// command is sent the user selected file is stored inside.
/// * `file_player` : `Option<play::Play>` is the Play object, which wrapps the Volume object, which
/// wrapps the FileReader object. These Russians nesting dolls divide the logic - implementation of
/// differents functions to play the song. Each Russian dolls owns the object of the smaller one,
/// which is usefull to avoid ownership problems (especially when using references inside structures).
/// * `is_playing` : `bool` is true when the stream is playing, false otherwise. 
/// * `nb_samples` : `Option<Arc<AtomicU64>>` is a thread-safe reference pointer created when the
/// FileReader object is created. It's then cloned, and give to UICommandsReceiver for the progress
/// bar calculs. `AtomicU64` is better (simpler) than `Mutex` but only for u64 type. 
/// * `total_samples` : `Arc<Option<u64>>` is a thread-safe reference pointer created when the
/// FileReader is created. It's then cloned and give to UICommandsReceiver, again for the progress
/// bar. This is not a Mutex of any type because this value does not change. 
///
/// # Exemple :
/// let ui_weak = ui.as_weak();
/// let mut receiver = slint_logic::UICommandsReceiver::new(ui_weak); 
/// 
/// [^note]: A Weak reference does not know if the Application is still running.
/// It implements Clone traits, allowing us to give a reference to this structure
/// When something needs to be done or change on the UI, this reference is upgraded and the instruction is sent via Slint functions (see `UICommandsReceiver::match_function()`)
pub struct UICommandsReceiver{
    files: Vec<String>, // new at creation, filled when fetch_directory is called. this must ensure
    // that it's defined when the user select an item from the list (because the popup automatically
    // call fetch_directory)
    ui: slint::Weak<AppWindow>,
    selected_file: String,
    file_player: Option<Play>,
    is_playing: bool,
    nb_samples: Option<Arc<AtomicU64>>,
    total_samples: Arc<Option<u64>>,
    volume_handle: Option<Arc<Mutex<f32>>>,
    creating_sinwave: String,
}

/// Implementation of the UICommandsReceiver structure
impl UICommandsReceiver{
    
    /// UICommandsReceiver constructor.
    ///
    /// # Arguments:
    ///
    /// * `files` : `Vec<String>` and more precisely [""] 
    /// * `ui` : `slint::Weak<AppWindow>` reference, which is a Weak[^note] version of the App
    /// reference, which can be clone.
    /// * `selected_file` : an empty `String` until user selects a file.
    /// * `file_player` : `None` until user selects a file. 
    /// * `is_playing` : `false` until user plays a song.
    /// * `nb_samples` `None` until the user selects a song.
    /// * `total_samples` : `Arc<None>` until the user selects a song. 
    /// * `volume_handle` : `None` until the user selects a song.
    /// 
    ///
    /// # Returns : `UICommandsReceiver` (Self)
    ///
    /// # Exemple : 
    /// ```ignore
    /// # use Audio_Player::{AppWindow, slint_logic};
    /// let ui = AppWindow::new().unwrap();
    /// let ui_weak = ui.as_weak();
    /// let mut receiver = slint_logic::UICommandsReceiver::new(ui_weak); 
    /// ```
    ///
    /// [^note]: A Weak reference does not know if the Application is still running.
    /// It implements Clone traits, allowing us to give a reference to this structure
    /// When something needs to be done or change on the UI, this reference is upgraded and the instruction is sent via Slint functions (see `UICommandsReceiver::match_function()`)
    pub fn new(ui: slint::Weak<AppWindow>) -> Self{
        Self {
            files: vec![],
            ui : ui,
            selected_file: String::new(),
            file_player: None,
            is_playing: false,
            nb_samples: None,
            total_samples: Arc::new(None),
            volume_handle: None,
            
            creating_sinwave: String::new(),
        }
    }


    /// Main Logic function. 
    ///
    /// The `mpsc::Receiver` is looping in a thread, computing this function each times it receives
    /// a message through the communication channel.
    ///
    /// The method `upgrade_in_event_loop` is implemented to send the Slint data from the backend to
    /// the front. This method is used here instead of `invoke_from_event_loop` because this one
    /// takes a Weak reference, upgrades it, and then do the same as the other, which don't upgrade.
    ///
    /// This uses pattern matching, which allows it to be modular and easy to upgrade. . 
    ///
    /// # Arguments: 
    ///
    /// * `command` : `UICommands` one of the various commands defined above (see `UICommands`)
    /// 
    /// # Few docs available :
    ///
    /// - [**invoke_from_event_loop**](https://docs.rs/slint/latest/slint/fn.invoke_from_event_loop.html)
    /// - [**Add an item to StandardListViewItem**](https://github.com/slint-ui/slint/discussions/2329)
    ///
    /// # Pattern Matching options :
    ///  
    /// - `UICommands::FetchDirectory` will fetch the list of files (and
    /// optionnaly the other directories recursively) inside the folder ./data_samples, using the
    /// `fetch_directory` function. 
    /// - `UICommands::ClickedInsidePopup()` occurs when the user click on the "Confirm" button of the
    /// popup which appears when the user choose to import a song from the project (and not from
    /// their computer).  
    /// - `UICommands::SwitchPlayMode()`occurs when the Play/Stop button is pressed by the user. The
    /// problem is that they can press this button for multiple uses. It need to check weither the
    /// stream was already playing (user paused) or not, in order to start it from the beginning or
    /// where the user paused it.
    /// - `UICommands::ImportFile()` occurs when the user wants to import a file from it's computer
    /// instead of playing the 2 availables inside the projetc. It uses the `rfd` library that
    /// implements a FileDialog object that opens a dialog popup for the user to choose a file. The
    /// extensions can be filtered, so for this project it only display .mp3, .wav et .ogg files.
    /// It returns the path of the file, and re-creates it inside the ./data_samples folder.
    /// - `UICommands::StopSamples()` will stop the stream by deleting the object
    /// ``file_reader` and calling `load_file` in order to recreate the Russian nesting dolls of 
    /// objects (FileReader, Volume and Play)
    ///
    /// # Exemple : 
    /// ```ignore
    /// # use std::sync::mspc;
    /// # use std::thread;
    /// # use Audio_Player::{AppWindow, slint_logic}; 
    /// // rx is a mpsc::Receiver, which receives message from mpsc::Sender 
    /// let (, rx) = mpsc::channel::<slint_logic::UICommands>();
    ///
    /// let ui = AppWindow::new().unwrap();
    /// let ui_weak = ui.as_weak();
    /// let mut receiver = slint_logic::UICommandsReceiver::new(ui_weak);
    ///
    /// // Here the receiver is polling until the application is stopped
    /// thread::spawn(move ||{
    /// while let Ok(msg) = rx.recv() {
    ///     receiver.match_command(msg);
    ///     }
    /// });     
    /// ```
    ///
    pub fn match_command(&mut self, command :UICommands){
        match command {

            UICommands::FetchDirectory() => {

                debug!("FetchDirectory called");
 
                self.files = vec![];

                // The user can either load a file from it's computer (with UICommands::ImportFile()) 
                // or choose a file from the ./data_samples directory of this project
                // There is 2 songs in this directory, in order for the user to test the audio
                // player easilly 
                let start_dir = Path::new("./data_samples");
                
                self.fetch_directory(start_dir).unwrap();
                let files_cpy = self.files.clone();
                
                /// `SharedString` is the Slint equivalent of the `String` type -- It implements From<String>
                /// The VecModel are the Slint equivalents of Vec 
                self.ui.upgrade_in_event_loop(move |ui| {
                    let items : Vec<StandardListViewItem> = files_cpy
                        .into_iter()
                        .map(|s| slint::StandardListViewItem::from(slint::SharedString::from(s)))
                        .collect();

                    let items_model = Rc::new(slint::VecModel::from(items));
                    ui.set_file_model(items_model.into());
                }); 
            }
            
            UICommands::ClickedInsidePopup(index) => {

                self.selected_file = match self.files.get(index as usize){
                    Some(file) => format!("File selected : {}", file), 
                    _ => format!("Error while selecting file"), 
                };
                
                let display = self.selected_file.clone();
                
                self.ui.upgrade_in_event_loop(move |ui| {
                    ui.set_selected_file(slint::SharedString::from(display));
                });

            }

            UICommands::LoadFile() => {

                debug!("LoadFile (enum) called");
                self.load_file();
                debug!("LoadFile (enum) finished");
            }

            UICommands::SwitchPlayMode() => {
                debug!("SwitchPlayMode called");
                // If this value is true, it means it starts playing 
                self.is_playing = !self.is_playing;

                let stream_playing = self.is_playing;
                if let Some(play_file) = self.file_player.as_mut() {

                    self.ui.upgrade_in_event_loop(move |ui|{
                        ui.set_is_playing(stream_playing);
                    });

                    if stream_playing  {
                        play_file.play_samples(); 
                    } else {
                       play_file.pause_samples(); 
                    }

                } 
            }

            UICommands::ImportFile() => {

                debug!("ImportFile called");
 
                // This allows the rfd to work on all OS (mainly Windows, macOs and Linux tho)
                #[cfg(not(target_arch = "wasm32"))]
                
                // rfd::FileDialog::pick_file returns a PathBuf
                let absolute_path = match rfd::FileDialog::new()
                    .add_filter("*", &["mp3", "wav", "ogg"])
                    .set_directory("~/")
                    .pick_file() {
                    Some(filePath) => filePath, 
                    _ => return,
                };
                
                // used to create the new file inside ./data_samples
                let filename = match absolute_path.file_name() {
                    Some(file) => file, 
                    _ => return, 
                }; 
                
                let filename_str = filename.to_str().unwrap();

                let project_path_full = format!("./data_samples/{}",filename_str);
                let project_path = Path::new(&project_path_full);
                fs::copy(&absolute_path, project_path).expect("Copy failed");

                // now self.selected_file needs to be changed for the other function to work.
                let _formated_project_path = format!("File selected : {}", project_path_full);
                self.selected_file = project_path_full;

                self.load_file();

            }

            UICommands::UpdateProgressBar() => {
                if self.nb_samples.as_ref().is_some() {
                    let nb_cur = self.nb_samples.as_ref().expect("nb_samples not computed").load(Ordering::SeqCst);
                    let nb_tot = self.total_samples.as_ref().expect("total_samples not computed"); 
                    let progress = (nb_cur as f64 / nb_tot as f64) as f32;

                    self.ui.upgrade_in_event_loop(move |ui|{
                       ui.set_progress_indicator(progress); 
                    });

                } 
            }
            
            UICommands::StopSamples() => {
                self.file_player = None;
                self.is_playing = false;
                self.load_file();
        
            }

            
            UICommands::ChangeVolume(value) => {
                if let Some(vol) = &self.volume_handle {
                    let mut v = vol.lock().unwrap();
                    *v = value / 50.0;
                }
            }

            UICommands::SendSinWave() => {
                let wave = Wave::new(0.5, 440.0, 0.0); 
                let source = SourceType::Oscillator(wave); 
                let volume = Volume::new(source,0.5); 
                self.file_player = Some(Play::new(volume));

                if let Some(file_p) = self.file_player.as_mut() {
                    file_p.play_samples();
                }
            }

            UICommands::SendSinGraph(height, width) => {
                
                println!("send_sin_graph");
                /**
                if (height == 0.0 || width == 0.0){
                    println!("singraph dimensions was 0");
                    return
                }; 


                println!("f32 height : {}\nf32 width : {}", height, width);
                let u_width = width as u32; 
                let u_height = height as u32; 
                
                println!("u32 height : {}\nu32 width : {}", u_width, u_height);

                let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(u_width as u32, u_height as u32);

                let mut pixmap = PixmapMut::from_bytes(
                    pixel_buffer.make_mut_bytes(), u_width, u_height
                ).unwrap();


                pixmap.fill(tiny_skia::Color::BLACK);
                
                // number of cols in the checkered bg 
                let n = 200;
                // rows 
                let m = 300;

                let cell_width = width / (n as f32) ; 
                let cell_height = height / (m as f32);

                let mut paint = Paint::default();
                paint.set_color_rgba8(50, 127, 150, 200);
                paint.anti_alias = true;

                let path = {
                    let mut pb = PathBuilder::new(); 
                    
                    let mut pb_width = cell_width; 
                    while pb_width < width as f32{
                        pb.move_to(pb_width, 0.0); 
                        pb.line_to(pb_width, height as f32);
                        pb_width += cell_width; 
                    }

                    let mut pb_height = cell_height; 
                    while pb_height < height as f32{
                        pb.move_to(0.0 , pb_height); 
                        pb.line_to(width as f32, pb_height);
                        pb_height += cell_height;
                    }

                    pb.close();
                    pb.finish().unwrap()
                };


                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );


                self.ui.upgrade_in_event_loop(move |ui|{
                    let image = Image::from_rgba8_premultiplied(pixel_buffer);
                    ui.set_singraph(image); 
                });
                */

                let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(640, 480);

                let mut pixmap = tiny_skia::PixmapMut::from_bytes(
                    pixel_buffer.make_mut_bytes(), 640, 480
                ).unwrap();
                pixmap.fill(tiny_skia::Color::TRANSPARENT);

                let mut paint1 = Paint::default();
                paint1.set_color_rgba8(50, 127, 150, 200);
                paint1.anti_alias = true;

                let mut paint2 = Paint::default();
                paint2.set_color_rgba8(220, 140, 75, 180);
                paint2.anti_alias = false;

                let path1 = {
                    let mut pb = PathBuilder::new();
                    pb.move_to(60.0, 60.0);
                    pb.line_to(160.0, 940.0);
                    pb.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
                    pb.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
                    pb.close();
                    pb.finish().unwrap()
                };

                let path2 = {
                    let mut pb = PathBuilder::new();
                    pb.move_to(940.0, 60.0);
                    pb.line_to(840.0, 940.0);
                    pb.cubic_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
                    pb.cubic_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
                    pb.close();
                    pb.finish().unwrap()
                };

                pixmap.fill_path(
                    &path1,
                    &paint1,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
                pixmap.fill_path(
                    &path2,
                    &paint2,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );


                self.ui.upgrade_in_event_loop(move |ui|{
                    let image = Image::from_rgba8(pixel_buffer);
                    ui.set_singraph(image); 
                });
            }

            UICommands::Quit() => {
                slint::quit_event_loop();     
            }

            _ => println!("Command not implemented. Received command is {:?}", command), 
        }
        
    }
    
    /// Method called in UICommandsReceiver::match_command
    /// 
    /// # Arguments : 
    /// * `dir` : Path reference indicates the directory to fetch 
    ///
    /// # Returns : 
    /// * `io::Result<()>` classic return.
    ///
    /// # Exemples :
    /// `self.fetch_directory("./")` will return a list of all the directories and their
    /// files.inside the root directory of this project.`
    fn fetch_directory(&mut self, dir: &Path) -> io::Result<()>{

        debug!("fetch_directory called with Path : {}", dir.display()); 

        for entry in fs::read_dir(dir)? {
            let entry = entry?; 
            let path = entry.path();
            self.files.push(format!("{}", path.display()));
            if path.is_dir() {
                self.fetch_directory(&path)?;
            } 
        }
        
        Ok(())
        
    }
    
    /// Function called to create all the Objects (Play, Volume and FileReader) required to handle a
    /// song 
    ///
    /// # Exemple : 
    /// `self.load_file()` will create a FileReader for the `self.selected_file`, a Volume to wrapp
    /// the FileReader and a Play to wrap the Volume. It'll finally update the title of the song
    /// inside the UI. 
    fn load_file(&mut self) {
        
        self.is_playing = false; 

        debug!("load_file (function) called");
        let display = self.selected_file.clone();
        let trimmed_file = self.selected_file.strip_prefix("File selected : ").unwrap_or(&self.selected_file).to_string();
        debug!("LoadFile : Trimmed File is {}",trimmed_file);

        let file_reader = FileReader::new(trimmed_file);

        let total_samples = file_reader.as_ref().expect("file_reader not initialized").get_total_samples();
        let nb_samples = file_reader.as_ref().expect("file reader not initialized").get_nb_samples();
        
        self.nb_samples = Some(nb_samples); 
        self.total_samples = total_samples;


        let source = SourceType::File(file_reader.expect("...")); 
        let volume = Volume::new(source,0.5);
        self.file_player = Some(Play::new(volume));

        self.ui.upgrade_in_event_loop(move |ui| {
            ui.set_selected_file_title(slint::SharedString::from(display));
            }
        );
    } 

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppWindow, fileReader, volume, slint_logic};  
    


    
    
    use slint::Model;
    
    

    use std::{
        sync::{
            mpsc,
            Arc,
        }, 
        fs::{self}, 
        path::Path,
    };
    
    use slint::Weak;

    fn setup() -> (Weak<AppWindow>, mpsc::Sender<UICommands>, mpsc::Receiver<UICommands>){
        let (tx, rx) = mpsc::channel::<UICommands>();
        // It's forbidden in Slint to try to launch multiple event loop 
        // Creating a Weak reference does not launch this loop 
        // The way of doing so in this project is not test-safe because it creates a weak reference
        // from a real AppWindow, which launch a event loop. 
        // As there is a couple of tests, the second one will always crash because of that loop. 
        let ui_weak = slint::Weak::<AppWindow>::default();; 
        (ui_weak, tx, rx)
    }

    #[test]
    fn test_init_receiver() {
        let (ui_weak, _, _rx) = setup(); 
        let receiver = UICommandsReceiver::new(ui_weak);
        assert!(receiver.files.is_empty());
        assert!(receiver.selected_file.is_empty());
        assert!(receiver.file_player.is_none()); 
        assert!(!receiver.is_playing); 
        assert!(receiver.nb_samples.is_none());
        assert_eq!(receiver.total_samples, Arc::new(None));
    }

    // cannot tst the init of the sender because it only stores ui_weak/tx clones 

    /// the idea of this test is to create a full directory with some files inside, and then delete
    /// all this directory, in order to avoid mocks or test directory inside the project 
    #[test] 
    fn test_fetch_directory() {
        
        let (ui_weak, _, _rx) = setup(); 
        let mut receiver = slint_logic::UICommandsReceiver::new(ui_weak);

        let test_dir = Path::new("./test_dir"); 
        fs::create_dir(test_dir);
        fs::File::create(test_dir.join("test_1.mp3")); 
        fs::File::create(test_dir.join("test_2.wav"));
        let sub_dir_path = test_dir.join("sub_dir");
        let sub_dir = Path::new(&sub_dir_path); 
        fs::create_dir(sub_dir); 
        fs::File::create(sub_dir.join("test_3.ogg")); 

        receiver.fetch_directory(test_dir);

        assert!(receiver.files.iter().any(|elt| elt.contains("test_1.mp3")));
        assert!(receiver.files.iter().any(|elt| elt.contains("test_2.wav")));
        assert!(receiver.files.iter().any(|elt| elt.contains("test_3.ogg"))); 

        fs::remove_dir_all(test_dir);
    }

}

