use dimple_core::model::{Artist, Entity, Medium, Model, Recording, RecordingSource, Release, ReleaseGroup, Track};
use rayon::iter::{ParallelBridge, ParallelIterator};
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{self, MetadataOptions, StandardTagKey, Tag}, probe::Hint, util::bits::contains_ones_byte_u16};
use walkdir::{WalkDir, DirEntry};

use std::{collections::{HashMap, HashSet}, fs::{File, FileType}, os::unix::fs::MetadataExt, path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}, thread, time::Instant};

use dimple_librarian::{librarian::Librarian, plugin::{NetworkMode, Plugin}};

use dimple_core::db::Db;

use crate::media_file::MediaFile;

mod media_file;

#[derive(Clone)]
pub struct MediaFilesPlugin {
    librarian: Arc<Mutex<Option<Librarian>>>,
    directories: Arc<Mutex<HashSet<PathBuf>>>,
    sender: Sender<()>,
}

impl MediaFilesPlugin {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let plugin = Self {
            sender,
            librarian: Default::default(),
            directories: Default::default(),
        };
        {
            let plugin = plugin.clone();
            thread::spawn(move || {
                for _ in receiver.iter() {
                    plugin.scan();
                }
            });
        }
        plugin
    }

    pub fn monitor_directory(&self, path: &PathBuf) {
        self.directories.lock().unwrap().insert(path.to_path_buf());
        // TODO add file system based directory monitoring
        self.rescan()
    }

    /// Triggers a rescan ether now or after the current scan finishes.
    /// TODO interrupt current and restart
    pub fn rescan(&self) {
        if self.librarian.lock().unwrap().is_some() {
            self.sender.send(()).unwrap();
        }
    }

    fn scan(&self) {
        let directories = self.directories.lock().unwrap().clone();
        let db = self.librarian.lock().unwrap().clone();
        if db.is_none() {
            return;
        }
        let db = db.unwrap();
        let now = Instant::now();
        let mut count = 0;
        let rec_sources: HashMap<String, RecordingSource> = db.list(&RecordingSource::default().model(), None)
            .unwrap()
            .map(Into::<RecordingSource>::into)
            .map(|rec_source: RecordingSource| (rec_source.source_id.clone(), rec_source))
            .collect();
        for dir in directories {
            for dir_entry in WalkDir::new(dir).into_iter() {
                if dir_entry.is_err() { continue }

                let path = dir_entry.unwrap().into_path();
                if !path.is_file() { continue }

                // Find the matching RecordingSource in the Db, if any. For
                // now, if one is found we assume we've processed this file and
                // don't need to do it again. In the future we'll want to handle
                // merging updated data.
                let source_id = format!("dmfp://{}", path.to_str().unwrap_or_default());
                // TODO this needs to be a real query. Major perf. issue.
                // let rec_source = db.list(&RecordingSource::default().model(), None).unwrap()
                //     .map(Into::<RecordingSource>::into)
                //     .find(|rec_source| rec_source.source_id == source_id);
                let rec_source = rec_sources.get(&source_id).cloned();
                if rec_source.is_some() { continue }

                // If no RecordingSource is found, read the file and merge the
                // info.
                let media_file = MediaFile::new(&path);
                if media_file.is_err() { continue }
                let media_file = media_file.unwrap();

                let artist = media_file.artist();
                let release_group = media_file.release_group();
                let release = media_file.release();
                let medium = media_file.medium();
                let track = media_file.track();
                let recording = media_file.recording();
                let recording_source = media_file.recording_source();

                let librarian = self.librarian.lock().unwrap().clone();
                let librarian = librarian.unwrap();

                // Add last_updated_at to objects, use it to avoid writes
                // once merged.
                // Make find simple: name if we have it, musicbrainzid if we
                // have it, etc.

                librarian.merge(artist.model());

                // TODO images for each of the above?

                // TODO genres for each of the above?

                count += 1;
            }
        }
        log::info!("Scanned {} files in {}ms", count, now.elapsed().as_millis());
    }
}

impl Plugin for MediaFilesPlugin {
    fn init(&self, librarian: &Librarian) {
        *self.librarian.lock().unwrap() = Some(librarian.clone());
        self.rescan();
    }

    fn set_network_mode(&self, _network_mode: &NetworkMode) {
        // Don't care, local only.
    }
}

