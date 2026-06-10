use Audio_Player::fileReader::FileReader;

#[test]
fn test_filereader_new_file() {
    let reader = FileReader::new(String::from("data_samples/test.mp3"));
    assert!(reader.is_ok());
}

#[test]
fn test_filereader_invalid_file() {
    let reader = FileReader::new(String::from("test_fail.mp3"));
    assert!(reader.is_err());
}

#[test]
fn test_filereader_makes_samples() {
    let mut reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();
    assert!(reader.next().is_some());
}

#[test]
fn test_filereader_samples_in_range() {
    let mut reader = FileReader::new(String::from("data_samples/test.mp3")).unwrap();
    if let Some(sample) = reader.next() {
        assert!(sample >= -1.0 && sample <= 1.0);
    }
}