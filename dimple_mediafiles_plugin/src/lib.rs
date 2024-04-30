use dimple_core::model::{Entity, RecordingSource, Track};
use rayon::iter::{ParallelBridge, ParallelIterator};
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{self, MetadataOptions, StandardTagKey, Tag}, probe::Hint, util::bits::contains_ones_byte_u16};
use walkdir::{WalkDir, DirEntry};

use std::{collections::{HashMap, HashSet}, fs::{File, FileType}, os::unix::fs::MetadataExt, path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}, thread, time::Instant};

use dimple_librarian::{librarian::Librarian, plugin::{NetworkMode, Plugin}};

use dimple_core::db::Db;

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
        let rec_sources: Vec<RecordingSource> = db.list(&RecordingSource::default().model(), None).unwrap().map(Into::into).collect();
        let rec_sources_by_source_id: HashMap<String, RecordingSource> = rec_sources.iter()
            .map(|rec_source| (rec_source.source_id.clone(), rec_source.clone())).collect();
        for dir in directories {
            for dir_entry in WalkDir::new(dir).into_iter() {
                if dir_entry.is_err() { continue }
                let path = dir_entry.unwrap().into_path();
                if !path.is_file() { continue }
                let source_id = format!("dmfp://{}", path.to_str().unwrap_or_default());
                // let mut rec_source = RecordingSource::find_by_source_id(&db, &source_id).unwrap_or_else(|| {
                //     RecordingSource {
                //         source_id,
                //         ..Default::default()
                //     }
                // });
                let mut rec_source = rec_sources_by_source_id.get(&source_id).map_or_else(|| {
                    RecordingSource {
                        source_id,
                        ..Default::default()
                    }
                }, |rec_source| rec_source.clone());
                let rec_source: RecordingSource = db.insert(&rec_source.model()).unwrap().into();
                count += 1;
            }
        }
        log::info!("Scanned {} files in {}ms", count, now.elapsed().as_millis());
    }


    // fn read_tags(e: &DirEntry) -> anyhow::Result<MediaFile> {
    //     // TODO think more about to_string_lossy
    //     let path = e.path().to_string_lossy().to_string();
    //     let extension = e.path().extension().map(|f| f.to_string_lossy().to_string());

    //     let mut hint = Hint::new();
    //     if let Some(extension) = extension {
    //         hint.with_extension(&extension);
    //     }
        
    //     let media_source = File::open(&path)?;
    //     let media_source_stream =
    //         MediaSourceStream::new(Box::new(media_source), Default::default());

    //     // Use the default options for metadata and format readers.
    //     let meta_opts: MetadataOptions = Default::default();
    //     let fmt_opts: FormatOptions = Default::default();

    //     // Probe the media source.
    //     let mut probed = symphonia::default::get_probe()
    //         .format(&hint, media_source_stream, &fmt_opts, &meta_opts)?;

    //     // Get the instantiated format reader.
    //     let mut format = probed.format;

    //     // Collect all of the tags from both the file and format metadata
    //     let mut tags: Vec<Tag> = vec![];

    //     if let Some(metadata) = probed.metadata.get() {
    //         if let Some(metadata) = metadata.current() {
    //             tags.extend(metadata.tags().to_owned());
    //         }
    //     }

    //     let metadata = format.metadata();

    //     if let Some(metadata) = metadata.current() {
    //         tags.extend(metadata.tags().to_owned());
    //     }

    //     let details = FileDetails {
    //         path: path.clone(),
    //         tags,
    //     };

    //     Ok(details)
    // }
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

// impl Collection for FileLibrary {
//     fn name(&self) -> String {
//         format!("FileLibrary({:?})", self.paths)
//     }

//     fn available_offline(&self) -> bool {
//         true
//     }

//     fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
//         match (of_type, related_to) {
//             (Entities::MediaFile(_), None) => {
//                 let files = self.files.lock().unwrap().clone();
//                 let media_files: Vec<MediaFile> = files.values().map(Into::into).collect();
//                 let models: Vec<Entities> = media_files.into_iter().map(|m| Entities::MediaFile(m)).collect();
//                 Box::new(models.into_iter())
//             }
//             _ => {
//                 Box::new(vec![].into_iter())
//             }
//         }
//     }

//     fn stream(&self, _entity: &Entities) -> Option<Box<dyn Iterator<Item = u8>>> {
//         todo!()
//         // let files = self.files.lock().unwrap().clone();
//         // let file = files.values()
//         //     .find(|f| {
//         //         let ra: RecordingSource = (*f).into();
//         //         !ra.source_ids.is_disjoint(&_entity.source_ids())
//         //     })?;
//         // log::debug!("found {}", &file.path);
//         // Some(Box::new(std::fs::read(file.path.clone()).ok()?.into_iter()))
//     // }
// }

// // https://github.com/navidrome/navidrome/blob/master/scanner/mapping.go#L31
// impl From<&FileDetails> for MediaFile {
//     fn from(value: &FileDetails) -> Self {
//         MediaFile {
//             key: value.path.clone(),

//             // TODO in the future maybe this is, or includes, the sha
//             // source_ids: std::iter::once(value.path.clone()).collect(),

//             artist: value.get_tag_value(StandardTagKey::Artist),
//             artist_mbid: value.get_tag_value(StandardTagKey::MusicBrainzArtistId),

//             album: value.get_tag_value(StandardTagKey::Album),
//             album_mbid: value.get_tag_value(StandardTagKey::MusicBrainzAlbumId),
//             album_type_mb: value.get_tag_value(StandardTagKey::MusicBrainzReleaseType),

//             album_artist: value.get_tag_value(StandardTagKey::AlbumArtist),
//             album_artist_mbid: value.get_tag_value(StandardTagKey::MusicBrainzAlbumArtistId),

//             title: value.get_tag_value(StandardTagKey::TrackTitle),
//             recording_mbid: value.get_tag_value(StandardTagKey::MusicBrainzRecordingId),
//             release_track_mbid: value.get_tag_value(StandardTagKey::MusicBrainzReleaseTrackId),

//             genre: value.get_tag_value(StandardTagKey::Genre),

//             // mb_album_comment: value.get_tag_value(StandardTagKey::commen),

//             ..Default::default()
//         }
//     }
// }

// impl FileDetails {
//     pub fn get_tag_value(&self, key: StandardTagKey) -> Option<String> {
//         self.tags.iter().find_map(|t| {
//             if let Some(std_key) = t.std_key {
//                 if std_key == key {
//                     return Some(t.value.to_string())
//                 }
//             }
//             None
//         })
//     }
// }

// TODO none of these are complete
// impl From<&MediaFile> for Artist {
//     fn from(value: &MediaFile) -> Self {
//         Self {
//             name: value.get_tag_value(StandardTagKey::AlbumArtist),
//             source_ids: std::iter::once(value.path.clone()).collect(),
//             known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzAlbumArtistId) {
//                 Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
//                 _ => HashSet::default(),
//             },
//             ..Default::default()
//         }
//     }
// }

// // impl From<&FileDetails> for ReleaseGroup {
// //     fn from(value: &FileDetails) -> Self {
// //         Self {
// //             title: value.get_tag_value(StandardTagKey::Album),
// //             source_ids: std::iter::once(value.path.clone()).collect(),
// //             known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzReleaseGroupId) {
// //                 Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
// //                 _ => HashSet::default(),
// //             },
// //             ..Default::default()
// //         }
// //     }
// // }

// // impl From<&FileDetails> for Release {
// //     fn from(value: &FileDetails) -> Self {
// //         Self {
// //             title: value.get_tag_value(StandardTagKey::Album),
// //             source_ids: std::iter::once(value.path.clone()).collect(),
// //             known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzAlbumId) {
// //                 Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
// //                 _ => HashSet::default(),
// //             },
// //             ..Default::default()
// //         }
// //     }
// // }

// // impl From<&FileDetails> for Recording {
// //     fn from(value: &FileDetails) -> Self {
// //         Self {
// //             title: value.get_tag_value(StandardTagKey::TrackTitle),
// //             source_ids: std::iter::once(value.path.clone()).collect(),
// //             known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzRecordingId) {
// //                 Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
// //                 _ => HashSet::default(),
// //             },
// //             ..Default::default()
// //         }
// //     }
// // }

// // impl From<&FileDetails> for RecordingSource {
// //     fn from(value: &FileDetails) -> Self {
// //         let path: &Path = Path::new(&value.path);
// //         let extension: Option<String> = path.extension().map(|e| e.to_string_lossy().to_uppercase());
// //         Self {
// //             source_ids: std::iter::once(value.path.clone()).collect(),
// //             known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzRecordingId) {
// //                 Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
// //                 _ => HashSet::default(),
// //             },
// //             extension,
// //             ..Default::default()
// //         }
// //     }
// // }

