
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused,non_snake_case)]


use std::env;
use slint::{
    platform::update_timers_and_animations,
    ComponentHandle, 
};

use symphonia::core::errors;

use audio_player::{
    play::Play, 
    volume::Volume, 
    slint_logic, 
    fileReader, 
    wave,
    AppWindow,
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

use crate::slint_logic::UICommandsReceiver;


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

    let ui = AppWindow::new().unwrap();

    env_logger::init();

    let (tx, rx) = mpsc::channel::<slint_logic::UICommands>();
    info!(">> (main) multi-producer simple-consumer communication channel initialized");

    let ui_weak = ui.as_weak();
    
    let mut sender = slint_logic::UICommandsSender::new(ui_weak.clone(), tx.clone());
    let mut receiver = slint_logic::UICommandsReceiver::new(ui_weak); 

    thread::spawn(move ||{
        while let Ok(msg) = rx.recv() {
            receiver.match_command(msg);
        }
    });     

    info!(">> (main) Consumer is receiving inside it's thread"); 
    sender.setup_callbacks();
    info!(">> (main) Callbacks setted"); 
    ui.run().unwrap();
    info!(">> (main) UI Launched");
    Ok(())
}

