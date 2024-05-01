use dimple_core::model::{Artist, Entity, Recording, RecordingSource, Release, ReleaseGroup, Track};
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
        for dir in directories {
            for dir_entry in WalkDir::new(dir).into_iter() {
                if dir_entry.is_err() { continue }

                let path = dir_entry.unwrap().into_path();
                if !path.is_file() { continue }

                let source_id = format!("dmfp://{}", path.to_str().unwrap_or_default());

                // TODO this needs to be a real query. Major perf. issue.
                let rec_source = db.list(&RecordingSource::default().model(), None).unwrap()
                    .map(Into::<RecordingSource>::into)
                    .find(|rec_source| rec_source.source_id == source_id);
                if rec_source.is_some() { continue }

                let rec_source = db.insert(&RecordingSource {
                    source_id,
                    ..Default::default()
                }.model()).unwrap();

                let media_file = MediaFile::new(&path);
                if media_file.is_err() { continue }
                let media_file = media_file.unwrap();

                // TODO get associated recording, or create
                let recording = db.list(&Recording::default().model(), Some(&rec_source))
                    .unwrap()
                    .next()
                    .unwrap_or_else(|| {
                        let recording = db.insert(&Recording {
                            title: media_file.tag(StandardTagKey::TrackTitle),
                            ..Default::default()
                        }.model()).unwrap();
                        db.link(&recording, &rec_source).unwrap();
                        recording
                    });
                    
                // TODO get associated track, or create
                let track = db.list(&Track::default().model(), Some(&recording))
                    .unwrap()
                    .next()
                    .unwrap_or_else(|| {
                        let track = db.insert(&Track {
                            title: media_file.tag(StandardTagKey::TrackTitle),
                            ..Default::default()
                        }.model()).unwrap();
                        db.link(&track, &recording).unwrap();
                        track
                    });
                    
                // TODO get associated release, or create
                let release = db.list(&Release::default().model(), Some(&track))
                    .unwrap()
                    .next()
                    .unwrap_or_else(|| {
                        let release = db.insert(&Release {
                            title: media_file.tag(StandardTagKey::Album),
                            ..Default::default()
                        }.model()).unwrap();
                        db.link(&release, &track).unwrap();
                        release
                    });
                    
                // TODO get associated release group, or create
                let release_group = db.list(&ReleaseGroup::default().model(), Some(&release))
                    .unwrap()
                    .next()
                    .unwrap_or_else(|| {
                        let release_group = db.insert(&ReleaseGroup {
                            title: media_file.tag(StandardTagKey::Album),
                            ..Default::default()
                        }.model()).unwrap();
                        db.link(&release_group, &release).unwrap();
                        release_group
                    });
                    
                // TODO get associated artist, or create
                let artist = db.list(&Artist::default().model(), Some(&release_group))
                    .unwrap()
                    .next()
                    .unwrap_or_else(|| {
                        let artist = db.insert(&Artist {
                            name: media_file.tag(StandardTagKey::Artist),
                            ..Default::default()
                        }.model()).unwrap();
                        db.link(&artist, &release_group).unwrap();
                        artist
                    });
                    
                // TODO images for each of the above
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

