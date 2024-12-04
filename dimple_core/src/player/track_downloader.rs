use std::{collections::HashMap, io::Cursor, sync::{Arc, RwLock}};

use playback_rs::{Hint, Song};

use crate::{library::Library, model::Track};

#[derive(Clone, Debug)]
pub enum TrackDownloadStatus {
    Downloading,
    Ready(Song),
    Error(String),
}

#[derive(Clone, Default)]
pub struct TrackDownloader {
    track_key_status: Arc<RwLock<HashMap<String, TrackDownloadStatus>>>,
}

impl TrackDownloader {
    pub fn get(&self, track: &Track, library: &Library) -> TrackDownloadStatus {
        let track = track.clone();
        let track_key = track.key.clone().unwrap();
        let mut track_key_status = self.track_key_status.write().unwrap();
        match track_key_status.get(&track_key) {
            Some(status) => status.clone(),
            None => {
                track_key_status.insert(track_key.clone(), TrackDownloadStatus::Downloading);
                let track_key_status = self.track_key_status.clone();
                let library = library.clone();
                std::thread::spawn(move || {
                    log::info!("downloading track {:?}", track.key);
                    let content = library.load_track_content(&track).expect("No valid sources found.");
                    let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();
                    log::info!("finished downloading track {:?}", track.key);
                    track_key_status.write().unwrap().insert(track_key, TrackDownloadStatus::Ready(song));
                });
                TrackDownloadStatus::Downloading
            }
        }
    }
}

