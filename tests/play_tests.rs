use Audio_Player::fileReader::FileReader;
use Audio_Player::volume::Volume;
use Audio_Player::play::Play;
use std::sync::{Arc, Mutex};

#[test]

fn test_play_new() {

    let reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();

    let vol = Arc::new(Mutex::new(0.5f32));

    let volume = Volume::new(reader, Arc::clone(&vol));

    let player = Play::new(volume);

    assert_eq!(*player.get_volume_handle().lock().unwrap(), 0.5f32);

}



#[test]

fn test_play_volume_can_be_modified() {

    let reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();

    let vol = Arc::new(Mutex::new(0.5f32));

    let volume = Volume::new(reader, Arc::clone(&vol));

    let player = Play::new(volume);

    assert_eq!(*player.get_volume_handle().lock().unwrap(), 0.5f32);

    let handle = player.get_volume_handle();

    *handle.lock().unwrap() = 0.9f32;

    assert_eq!(*handle.lock().unwrap(), 0.9f32);

}
