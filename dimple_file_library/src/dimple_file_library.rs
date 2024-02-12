
use std::{collections::{HashMap, HashSet}, error::Error, fs::File, ops::Deref, sync::{Arc, Mutex}, time::{Duration, Instant}};
use dimple_core::{collection::Collection, model::{Artist, KnownId, Recording, RecordingSource, Release}};
use dimple_core::model::Model;
use rayon::iter::{ParallelBridge, ParallelIterator};
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag}, probe::Hint};
use walkdir::{WalkDir, DirEntry};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct FileLibrary {
    paths: Vec<String>,
    files: Arc<Mutex<HashMap<String, FileDetails>>>,
}

impl FileLibrary {
    pub fn new(paths: &[String]) -> Self {
        let files = Arc::new(Mutex::new(HashMap::new()));
        for path in paths {
            let path = path.clone();
            let files = files.clone();
            std::thread::spawn(move || Self::scanner(&path, files));
        }
        Self {
            paths: paths.into(),
            files,
        }
    }

    fn scanner(path: &str, files: Arc<Mutex<HashMap<String, FileDetails>>>) {
        loop {
            let now = Instant::now();
            WalkDir::new(path)
                .into_iter()
                .par_bridge()
                .filter(|e| e.is_ok())
                .map(|e| e.unwrap())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().is_some())
                .map(|e| Self::read(&e).ok())
                .filter(|e| e.is_some())
                .map(|e| e.unwrap())
                .for_each(|f| {
                    files.lock().unwrap().insert(f.path.clone(), f.clone());
                });

            log::info!("Scanned {} files in {}ms", files.lock().unwrap().len(), now.elapsed().as_millis());

            std::thread::sleep(Duration::from_secs(60 * 60));
        }
    }

    fn read(e: &DirEntry) -> Result<FileDetails, Box<dyn Error>> {
        let path = e.path().to_string_lossy().to_string();
        let extension = e.path().extension().map(|f| f.to_string_lossy().to_string());

        let mut hint = Hint::new();
        if let Some(extension) = extension {
            hint.with_extension(&extension);
        }
        
        let media_source = File::open(&path)?;
        let media_source_stream =
            MediaSourceStream::new(Box::new(media_source), Default::default());

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        // Probe the media source.
        let mut probed = symphonia::default::get_probe()
            .format(&hint, media_source_stream, &fmt_opts, &meta_opts)?;

        // Get the instantiated format reader.
        let mut format = probed.format;

        // Collect all of the tags from both the file and format metadata
        let mut tags: Vec<Tag> = vec![];

        if let Some(metadata) = probed.metadata.get() {
            if let Some(metadata) = metadata.current() {
                tags.extend(metadata.tags().to_owned());
            }
        }

        let metadata = format.metadata();

        if let Some(metadata) = metadata.current() {
            tags.extend(metadata.tags().to_owned());
        }

        let details = FileDetails {
            path: path.clone(),
            tags,
        };

        // log::info!("read {:?}", details);

        Ok(details)
    }
}

#[derive(Clone, Debug, Default)]
struct FileDetails {
    path: String,
    tags: Vec<Tag>,
}

impl Collection for FileLibrary {
    fn name(&self) -> String {
        format!("FileLibrary({:?})", self.paths)
    }

    fn list(&self, of_type: &Model, related_to: Option<&Model>) -> Box<dyn Iterator<Item = Model>> {
        match (of_type, related_to) {
            (Model::Artist(_), None) => {
                let files = self.files.lock().unwrap().clone();
                let artists: Vec<Artist> = files.values().map(Into::into).collect();
                let models: Vec<Model> = artists.iter().map(Artist::entity).collect();
                Box::new(models.into_iter())
            }
            (Model::Release(_), None) => {
                let files = self.files.lock().unwrap().clone();
                let releases: Vec<Release> = files.values().map(Into::into).collect();
                let models: Vec<Model> = releases.iter().map(Release::entity).collect();
                Box::new(models.into_iter())
            }
            (Model::Recording(_), None) => {
                let files = self.files.lock().unwrap().clone();
                let recordings: Vec<Recording> = files.values().map(Into::into).collect();
                let models: Vec<Model> = recordings.iter().map(Recording::entity).collect();
                Box::new(models.into_iter())
            }
            (Model::RecordingSource(_), Some(Model::Recording(recording))) => {
                // TODO trash test code, will be using sourceid??
                let files = self.files.lock().unwrap().clone();
                let sources: Vec<RecordingSource> = files.values()
                    .filter(|source| {
                        let source_rec: Recording = (*source).into();
                        source_rec == *recording
                    })
                    .map(Into::into)
                    .collect();
                let models: Vec<Model> = sources.iter().map(RecordingSource::entity).collect();
                Box::new(models.into_iter())
            }
            // (Model::Release(_), Some(Model::Artist(artist))) => {
            //     // TODO also trash, I think?
            //     let files = self.files.lock().unwrap().clone();
            //     let releases: Vec<Release> = files.values()
            //         .filter(|r| {
            //             let ra: Artist = (*r).into();
            //             ra == *artist
            //         })
            //         .map(Into::into)
            //         .collect();
            //     let models: Vec<Model> = releases.iter().map(Release::entity).collect();
            //     Box::new(models.into_iter())
            // }
            // (Model::Recording(_), Some(Model::Release(release))) => {
            //     // TODO also trash, I think?
            //     let files = self.files.lock().unwrap().clone();
            //     let recordings: Vec<Recording> = files.values()
            //         .filter(|rec| {
            //             let rec_rel: Release = (*rec).into();
            //             rec_rel == *release
            //         })
            //         .map(Into::into)
            //         .collect();
            //     let models: Vec<Model> = recordings.iter().map(Recording::entity).collect();
            //     Box::new(models.into_iter())
            // }
            _ => {
                Box::new(vec![].into_iter())
            }
        }
    }
}

impl FileDetails {
    pub fn get_tag_value(&self, key: StandardTagKey) -> Option<String> {
        self.tags.iter().find_map(|t| {
            if let Some(std_key) = t.std_key {
                if std_key == key {
                    return Some(t.value.to_string())
                }
            }
            None
        })
    }
}

// TODO none of these are complete
impl From<&FileDetails> for Artist {
    fn from(value: &FileDetails) -> Self {
        Self {
            name: value.get_tag_value(StandardTagKey::AlbumArtist),
            source_ids: std::iter::once(value.path.clone()).collect(),
            known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzAlbumArtistId) {
                Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
                _ => HashSet::default(),
            },
            ..Default::default()
        }
    }
}

impl From<&FileDetails> for Release {
    fn from(value: &FileDetails) -> Self {
        Self {
            title: value.get_tag_value(StandardTagKey::Album),
            source_ids: std::iter::once(value.path.clone()).collect(),
            known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzAlbumId) {
                Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
                _ => HashSet::default(),
            },
            ..Default::default()
        }
    }
}

impl From<&FileDetails> for Recording {
    fn from(value: &FileDetails) -> Self {
        Self {
            title: value.get_tag_value(StandardTagKey::TrackTitle),
            source_ids: std::iter::once(value.path.clone()).collect(),
            known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzRecordingId) {
                Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
                _ => HashSet::default(),
            },
            ..Default::default()
        }
    }
}

impl From<&FileDetails> for RecordingSource {
    fn from(value: &FileDetails) -> Self {
        Self {
            source_ids: std::iter::once(value.path.clone()).collect(),
            known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzRecordingId) {
                Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
                _ => HashSet::default(),
            },
            ..Default::default()
        }
    }
}

// for tag in tags {
//     if let Some(StandardTagKey::TrackTitle) = tag.std_key {
//         details.title = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::Album) = tag.std_key {
//         details.album = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::Artist) = tag.std_key {
//         details.artist = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::MusicBrainzAlbumId) = tag.std_key {
//         details.musicbrainz_release_id = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::MusicBrainzArtistId) = tag.std_key {
//         details.musicbrainz_artist_id = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::MusicBrainzRecordingId) = tag.std_key {
//         details.musicbrainz_recording_id = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::MusicBrainzReleaseGroupId) = tag.std_key {
//         details.musicbrainz_release_group_id = Some(tag.value.to_string());
//     }
//     else if let Some(StandardTagKey::MusicBrainzTrackId) = tag.std_key {
//         details.musicbrainz_track_id = Some(tag.value.to_string());

// pub trait Equivalent {
//     fn equivalent(&self, other: &Self) -> bool;
// }

// impl Equivalent for Artist {
//     fn equivalent(&self, other: &Self) -> bool {
//         let matcher = SkimMatcherV2::default();
//         if let (Some(l), Some(r)) = (self.name, other.name) {
//             if !l.is_empty() && !r.is_empty() {
//                 let _ = ;

//         }
//         todo!()
//     }
// }

// let matcher = SkimMatcherV2::default();
// let sources: Vec<Model> = recordings.iter()
//     .filter(|r2| matcher.fuzzy_match(&r.title, &r2.title).is_some())
//     .map(|r| RecordingSource {
//         known_ids: r.known_ids.clone(),
//         source_ids: r.source_ids.clone(),
//         ..Default::default()
//     })
//     .map(|r| r.entity())
//     .collect();


// fn compare(&self, other: &Self) -> f32 {
//     if self.key.is_some() && self.key == other.key {
//         1.0
//     }
//     else if !self.source_ids.is_disjoint(&other.source_ids) {
//         return 1.0
//     }

//     else if has_common_entry(&self.known_ids, &other.known_ids) {
//         return 1.0;
//     }
//     else if self.name == other.name {
//         return 0.25;
//     }
//     else {
//         return 0.0
//     }
// }
