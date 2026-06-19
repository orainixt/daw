
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused,non_snake_case)]


use std::env;


use symphonia::core::errors;

use audio_player::{
    sound_design::{
        file_reader, 
        play::Play, 
        volume::Volume, 
        wave, 
    }, 
    ui::{
        dancing_wave::{DancingWaves}, main_app::MainApp
    }
};
// crates 

use std::{
    path::PathBuf, 
    error::Error, 
    sync::{
        mpsc,
    },
    thread,
    rc::Rc
};
use log::{debug, error, log_enabled, info, Level};



/// main function
///
/// Starts 
///     - UI  
///     - MPSC (multi-producer simple-consumer) [@see](https://doc.rust-lang.org/std/sync/mpsc/)
///     - env-logger (implements info, debug etc. macros)
///     - Consumer thread (moving it outside main requires to add lifetime's parameters to ensure
///     it'll live long enough. not usefull if nothing else is added though)
///
/// @return : () or Box<dyn Error> Box uses dynamic dispatch for different types of Error
fn main() -> Result<(), Box<dyn Error>> {


    env_logger::init();
   /** 
    let (tx, rx) = mpsc::channel::<slint_logic::UICommands>();
    info!(">> (main) multi-producer simple-consumer communication channel initialized");
    
    let tx_cpy = tx.clone(); 




//    let mut sender = slint_logic::AppCommandSender::new(ui_weak.clone(), tx.clone());
//    let mut receiver = slint_logic::AppCommandHandler::new(ui_weak); 

    thread::spawn(move ||{
        while let Ok(msg) = rx.recv() {
            receiver.match_command(msg);
        }
    });     
    */


    let song_name = String::from("deadbeef.txt"); 

    let files_list = [
        "What does a piccolo sound like (Ode to Joy).mp3"
    ]; 

    let lfiles : Vec<String> = files_list
        .iter()
        .map(|i| format!("samples/{}", i))
        .collect();
    
    //this should be fetced by symphonia in file_reader
    let sample_rate = 48000.0;
    let fft_size : usize = 2048;

    let options = eframe::NativeOptions::default(); 

    eframe::run_native(
        "DAW", 
        options, 
        Box::new(|_creation_ctx| Ok(Box::new(MainApp::new(lfiles, sample_rate, fft_size, song_name)))),
    );

    info!(">> (main) UI Launched");

    Ok(())
}

