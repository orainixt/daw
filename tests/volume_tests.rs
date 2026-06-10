use Audio_Player::fileReader::FileReader;
use Audio_Player::volume::Volume;
use std::sync::{Arc, Mutex};

#[test]
fn test_volume_new() {
    let reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();
    let vol = Arc::new(Mutex::new(0.5f32));
    let volume = Volume::new(reader, Arc::clone(&vol));

    assert_eq!(*volume.get_volume().lock().unwrap(), 0.5);
}

#[test]
fn test_volume_produces_samples() {
    let reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();
    let vol = Arc::new(Mutex::new(0.5f32));
    let mut volume = Volume::new(reader, Arc::clone(&vol));

    assert!(volume.next().is_some());
}

#[test]
fn test_volume_silence_at_begining() {
    let reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();
    let vol = Arc::new(Mutex::new(0.0f32));
    let mut volume = Volume::new(reader, Arc::clone(&vol));

    if let Some(sample) = volume.next() {
        assert!(sample.abs() < 0.0001);
    }
}