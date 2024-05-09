use dimple_core::model::{Artist, Entity, Medium, Model, Recording, RecordingSource, Release, ReleaseGroup, Track};
use rayon::iter::{ParallelBridge, ParallelIterator};
use symphonia::core::{conv::IntoSample, formats::FormatOptions, io::MediaSourceStream, meta::{self, MetadataOptions, StandardTagKey, Tag}, probe::Hint, util::bits::contains_ones_byte_u16};
use walkdir::{WalkDir, DirEntry};

use std::{collections::{HashMap, HashSet}, fs::{File, FileType}, os::unix::fs::MetadataExt, path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}, thread, time::{Duration, Instant}};

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

    fn scan(&self) {
        let directories = self.directories.lock().unwrap().clone();
        let db = self.librarian.lock().unwrap().clone();
        if db.is_none() {
            return;
        }
        let db = db.unwrap();
        let now = Instant::now();
        let mut count = 0;
        let mut skipped = 0;
        // TODO see query note below. This is a temporary cache.
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
                // TODO this needs to be a real query. Major perf. issue. Caching for now.
                // let rec_source = db.list(&RecordingSource::default().model(), None).unwrap()
                //     .map(Into::<RecordingSource>::into)
                //     .find(|rec_source| rec_source.source_id == source_id);
                let rec_source = rec_sources.get(&source_id).cloned();

                // Compare last modified of the file and the rec source
                // and if the file is older, continue / skip.
                let mtime = path.metadata().unwrap().modified().unwrap();
                if let Some(rec_source) = &rec_source {
                    if let Some(last_modified) = rec_source.last_modified {
                        if last_modified >= mtime {
                            log::debug!("Skipping {:?}, {:?} {:?} is the same or newer than {:?}", 
                                path, 
                                rec_source.key, 
                                last_modified, 
                                mtime);
                            skipped += 1;
                            continue;
                        }
                    }
                }

                // Read the media file.
                let media_file = MediaFile::new(&path);
                if media_file.is_err() { continue }
                let media_file = media_file.unwrap();
            
                // Find a matching artist in the Db, if any, and merge it. If
                // there is no matching artist in the Db then we'll create one
                // as long as we have at least a name or mbid.
                // TODO currently broken cause we're not creating the rec source
                let mf_artist = media_file.artist();
                let mut artist: Option<Artist> = None;
                if let Some(db_artist) = Self::find_artist(&self, &db, &mf_artist) {
                    let merged = Artist::merge(db_artist, mf_artist);
                    artist = Some(db.insert(&merged.model()).unwrap().into());
                }
                else {
                    if mf_artist.name.is_some() || mf_artist.known_ids.musicbrainz_id.is_some() {
                        artist = Some(db.insert(&mf_artist.model()).unwrap().into());
                        log::debug!("Created artist: {:?}", 
                            artist.clone().unwrap().name
                        );
                    }
                }

                // Find a matching artist-release in the Db, if any, and merge
                // it. Same as above except now we include the artist as a
                // link requirement.
                let mf_release = media_file.release();
                let mut release: Option<Release> = None;
                if let Some(artist) = artist {
                    if let Some(db_release) = Self::find_artist_release(&self, &db, &artist, &mf_release) {
                        let merged = Release::merge(db_release, mf_release);
                        release = Some(db.insert(&merged.model()).unwrap().into());
                    }
                    else {
                        // TODO Ids
                        if mf_release.title.is_some() {
                            release = Some(db.insert(&mf_release.model()).unwrap().into());
                            log::debug!("Created artist release: {:?}-{:?}", 
                                artist.name,
                                release.clone().unwrap().title
                            );
                        }
                    }

                    if release.is_some() {
                        db.link(&release.clone().unwrap().model(), &artist.model()).unwrap();
                    }
                }

                // Find a matching release-track in the Db, if any, and merge
                // it. Same as above except now we include the release as a
                // link requirement.
                let mf_track = media_file.track();
                let mut track: Option<Track> = None;
                if let Some(release) = release {
                    if let Some(db_track) = Self::find_release_track(&self, &db, &release, &mf_track) {
                        let merged = Track::merge(db_track, mf_track);
                        track = Some(db.insert(&merged.model()).unwrap().into());
                    }
                    else {
                        // TODO Ids
                        if mf_track.title.is_some() {
                            track = Some(db.insert(&mf_track.model()).unwrap().into());
                            log::debug!("Created release track: {:?}-{:?}", 
                                release.title,
                                track.clone().unwrap().title
                            );
                        }
                    }

                    if track.is_some() {
                        db.link(&track.unwrap().model(), &release.model()).unwrap();
                    }
                }

                // TODO maybe patch in medium and release group after the fact?
                // or just rid of them?

                // let path = vec![artist.model(), release.model(), track.model()];
                // let path_result = db.find_path(path);

                count += 1;
            }
        }
        log::info!("Scanned {}, skipped {} in {}ms", 
            count, 
            skipped, 
            now.elapsed().as_millis());
    }

    fn find_artist(&self, db: &dyn Db, artist: &Artist) -> Option<Artist> {
        db.list(&Artist::default().model(), None).unwrap()
            .map(Into::<Artist>::into)
            .filter(|artist_opt| {
                artist.name.is_some() && artist.name == artist_opt.name
            })
            .next()
    }

    fn find_artist_release(&self, db: &dyn Db, artist: &Artist, release: &Release) -> Option<Release> {
        db.list(&Release::default().model(), Some(&artist.model())).unwrap()
            .map(Into::<Release>::into)
            .filter(|release_opt| {
                release.title.is_some() && release.title == release_opt.title
            })
            .next()
    }

    fn find_release_track(&self, db: &dyn Db, release: &Release, track: &Track) -> Option<Track> {
        db.list(&Track::default().model(), Some(&release.model())).unwrap()
            .map(Into::<Track>::into)
            .filter(|track_opt| {
                track.title.is_some() && track.title == track_opt.title
            })
            .next()
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

