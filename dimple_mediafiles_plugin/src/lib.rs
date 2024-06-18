use dimple_core::model::{Artist, Entity, Genre, Medium, Model, Dimage, RecordingSource, Release, ReleaseGroup, Track};
use rayon::iter::{ParallelBridge, ParallelIterator};
use walkdir::WalkDir;

use std::{collections::HashSet, path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}, thread, time::Instant};

use dimple_librarian::{librarian::Librarian, merge::Merge, plugin::{NetworkMode, Plugin}};

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

    // TODO query performance
    // TODO recording and rec source
    fn scan(&self) {
        let directories = self.directories.lock().unwrap().clone();
        let db = self.librarian.lock().unwrap().clone();
        if db.is_none() {
            return;
        }
        let db = db.unwrap();
        let now = Instant::now();
        for dir in directories {            
            WalkDir::new(dir).into_iter().par_bridge().for_each(|dir_entry| {
                if dir_entry.is_err() { return }

                let path = dir_entry.unwrap().into_path();
                if !path.is_file() { return }

                // Self::scan_path(&db, &path);
            });
        }
        // log::info!("Scanned {}, skipped {} in {}ms", 
        //     count, 
        //     skipped, 
        //     now.elapsed().as_millis());
        log::info!("Scan complete.");
    }

    fn scan_path(db: &dyn Db, path: &PathBuf) {
        // Find the matching RecordingSource in the Db, if any.
        let source_id = format!("dmfp://{}", path.to_str().unwrap_or_default());
        let rec_source = db.list(&RecordingSource::default().model(), &None).unwrap()
            .map(Into::<RecordingSource>::into)
            .find(|rec_source| rec_source.source_id == source_id);

        // Compare last modified of the file and the rec source
        // and if the file is older, continue / skip.
        // TODO currently broken cause we're not creating the rec source
        let mtime = path.metadata().unwrap().modified().unwrap();
        if let Some(rec_source) = &rec_source {
            if let Some(last_modified) = rec_source.last_modified {
                if last_modified >= mtime {
                    log::debug!("Skipping {:?}, {:?} {:?} is the same or newer than {:?}", 
                        path, 
                        rec_source.key, 
                        last_modified, 
                        mtime);
                    // skipped += 1;
                    // continue;
                    return
                }
            }
        }

        // Read the media file.
        let media_file = MediaFile::new(&path);
        if media_file.is_err() { return }
        let media_file = media_file.unwrap();
    
    }
}

impl Plugin for MediaFilesPlugin {
    fn name(&self) -> String {
        "MediaFilesPlugin".to_string()
    }    
}


