use crate::{

    sound_design::{
        file_reader::FileReader,
        wave::Wave,
    }
};


use std::{
    sync::{
        Arc,
        atomic::Ordering,
    }, 
}; 

use atomic_float::AtomicF32; 

pub enum SourceType {
    File(FileReader),
    Oscillator(Wave),
}

impl Iterator for SourceType {
    type Item = f32; 

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SourceType::File(fileReader) => fileReader.next(),
            SourceType::Oscillator(wave) => wave.next(),
        }
    }
}

/// Structure for the volume contains the audio file to play and a shared volume 
pub struct Volume {
    source: SourceType,
    vol: Arc<AtomicF32>,
}

/// An iterator that implements a function next that associate each sample with the volume value 
impl Iterator for Volume {
    type Item = f32; 

    /// Returns the next sample multiplied by the current volume
    /// # Returns
    /// * Some(new_s) if the sample has a next 
    /// None otherwise

    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    /// use Audio_Player::volume::Volume;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let reader = FileReader::new(String::from("song.mp3")).unwrap();
    /// let vol = Arc::new(Mutex::new(0.5f32));
    /// let mut volume = Volume::new(reader, Arc::clone(&vol));
    ///
    /// // next() returns Some(f32) with the new volume
    /// // or None if the file ended
    /// while let Some(sample) = volume.next() {
    ///     println!("sample avec volume : {}", sample);
    /// }
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.source.next() {
            let curr_v = self.vol.load(Ordering::Relaxed);
            let new_s = sample * curr_v;       
            Some(new_s)
        } else {None}
    }
}


impl Volume {
    /// Create a new Volume that contains the file audio to play with the volume value
    /// # Arguments :
    /// * `file` a fileReader that will be played
    /// * `vol` `Arc<Mutex<f32>>` a shared volume
    /// # Returns
    /// * `Volume` a volume Type that contains the file audio with the volume 
    
    /// # Examples
    /// ```no_run
    /// use Audio_Player::fileReader::FileReader;
    /// use Audio_Player::volume::Volume;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let reader = FileReader::new(String::from("song.mp3")).unwrap();
    ///
    /// // We create a volume shared with 0.5 value (50% of the volume)
    /// let vol = Arc::new(Mutex::new(0.5f32));
    ///
    /// // We create the Volume with the file and the shared volume 
    /// let volume = Volume::new(reader, Arc::clone(&vol));
    /// ```
    pub fn new(source: SourceType, vol: f32) -> Volume{
        Volume{
            source : source,
            vol : Arc::new(AtomicF32::new(vol)),
        }
    }

    /// Gives a cloned volume so we can modify it in another code
    /// # Returns
    /// `Arc<Mutex<f32>>` a cloned volume 
    
    /// # Examples
    /// ```no_run
    /// use Audio_Player::volume::Volume;
    /// use std::sync::{Arc, Mutex};
    /// use Audio_Player::fileReader::FileReader;
    ///
    /// let reader = FileReader::new(String::from("song.mp3")).unwrap();
    /// let vol = Arc::new(Mutex::new(0.5f32));
    /// let volume = Volume::new(reader, Arc::clone(&vol));
    ///
    /// // get_volume returns a shared reference to the volume 
    /// // And so we can modify the volume from anywhere in the code
    /// let volume_ref = volume.get_volume();
    /// let mut v = volume_ref.lock().unwrap();
    /// *v = 0.8; // we change the volume to 80%
    /// ```
    pub fn get_volume(&self) -> Arc<AtomicF32>{
        Arc::clone(&self.vol)
    }

}
