use std::{io::Cursor, num::NonZeroUsize, sync::{Arc, RwLock}};

use lru::LruCache;
use playback_rs::{Hint, Song};
use threadpool::ThreadPool;

use crate::{library::Library, model::Track};

#[derive(Clone, Debug)]
pub enum TrackDownloadStatus {
    Downloading,
    Ready(Song),
    Error(String),
}

#[derive(Clone)]
pub struct TrackDownloader {
    cache: Arc<RwLock<LruCache<String, TrackDownloadStatus>>>,
    threadpool: ThreadPool,
}

impl Default for TrackDownloader {
    fn default() -> Self {
        Self { 
            // TODO magic, just felt right, needs to handle multiple downloads in
            // progress. Probably should be related to threadpool len.
            cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(5).unwrap()))), 
            threadpool: Default::default() 
        }
    }
}

impl TrackDownloader {
    pub fn get(&self, track: &Track, library: &Library) -> TrackDownloadStatus {
        let cache = self.cache.clone();
        let track = track.clone();
        let library = library.clone();
        self.cache.write().unwrap().get_or_insert(track.key.clone().unwrap(), move || {
            self.threadpool.execute(move || {
                let content = library.load_track_content(&track).expect("No valid sources found.");
                let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();
                cache.write().unwrap().put(track.key.clone().unwrap(), TrackDownloadStatus::Ready(song));
            });
            TrackDownloadStatus::Downloading
        }).clone()
    }
}

