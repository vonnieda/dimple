
// use std::{collections::{HashMap, HashSet}, error::Error, fs::File, ops::Deref, path::Path, sync::{Arc, Mutex}, time::{Duration, Instant}};
// use dimple_core::{collection::Collection, model::{Artist, Entity, KnownId, MediaFile, Recording, RecordingSource, Release, ReleaseGroup}};
// use dimple_core::model::Entities;
// use rayon::iter::{ParallelBridge, ParallelIterator};
// use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag}, probe::Hint};
// use walkdir::{WalkDir, DirEntry};

// pub struct FileLibrary {
//     paths: Vec<String>,
//     files: Arc<Mutex<HashMap<String, FileDetails>>>,
// }

// #[derive(Clone, Debug, Default)]
// struct FileDetails {
//     path: String,
//     tags: Vec<Tag>,
// }

// impl FileLibrary {
//     pub fn new(paths: &[String]) -> Self {
//         let files = Arc::new(Mutex::new(HashMap::new()));
//         for path in paths {
//             let path = path.clone();
//             let files = files.clone();
//             std::thread::spawn(move || Self::scanner(&path, files));
//         }
//         Self {
//             paths: paths.into(),
//             files,
//         }
//     }

//     fn scanner(path: &str, files: Arc<Mutex<HashMap<String, FileDetails>>>) {
//         loop {
//             let now = Instant::now();
//             WalkDir::new(path)
//                 .into_iter()
//                 .par_bridge()
//                 .filter(|e| e.is_ok())
//                 .map(|e| e.unwrap())
//                 .filter(|e| e.file_type().is_file())
//                 .filter(|e| e.path().extension().is_some())
//                 .map(|e| Self::read(&e).ok())
//                 .filter(|e| e.is_some())
//                 .map(|e| e.unwrap())
//                 .for_each(|f| {
//                     files.lock().unwrap().insert(f.path.clone(), f.clone());
//                 });

//             log::info!("Scanned {} files in {}ms", files.lock().unwrap().len(), now.elapsed().as_millis());

//             // TODO also monitor dir
//             std::thread::sleep(Duration::from_secs(60));
//         }
//     }

//     fn read(e: &DirEntry) -> Result<FileDetails, Box<dyn Error>> {
//         // TODO think more about to_string_lossy
//         let path = e.path().to_string_lossy().to_string();
//         let extension = e.path().extension().map(|f| f.to_string_lossy().to_string());

//         let mut hint = Hint::new();
//         if let Some(extension) = extension {
//             hint.with_extension(&extension);
//         }
        
//         let media_source = File::open(&path)?;
//         let media_source_stream =
//             MediaSourceStream::new(Box::new(media_source), Default::default());

//         // Use the default options for metadata and format readers.
//         let meta_opts: MetadataOptions = Default::default();
//         let fmt_opts: FormatOptions = Default::default();

//         // Probe the media source.
//         let mut probed = symphonia::default::get_probe()
//             .format(&hint, media_source_stream, &fmt_opts, &meta_opts)?;

//         // Get the instantiated format reader.
//         let mut format = probed.format;

//         // Collect all of the tags from both the file and format metadata
//         let mut tags: Vec<Tag> = vec![];

//         if let Some(metadata) = probed.metadata.get() {
//             if let Some(metadata) = metadata.current() {
//                 tags.extend(metadata.tags().to_owned());
//             }
//         }

//         let metadata = format.metadata();

//         if let Some(metadata) = metadata.current() {
//             tags.extend(metadata.tags().to_owned());
//         }

//         let details = FileDetails {
//             path: path.clone(),
//             tags,
//         };

//         Ok(details)
//     }
// }

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

// // // TODO none of these are complete
// // impl From<&FileDetails> for Artist {
// //     fn from(value: &FileDetails) -> Self {
// //         Self {
// //             name: value.get_tag_value(StandardTagKey::AlbumArtist),
// //             source_ids: std::iter::once(value.path.clone()).collect(),
// //             known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzAlbumArtistId) {
// //                 Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
// //                 _ => HashSet::default(),
// //             },
// //             ..Default::default()
// //         }
// //     }
// // }

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

