
use std::{collections::{HashMap, HashSet}, error::Error, fs::File, ops::Deref, path::Path, sync::{Arc, Mutex}, time::{Duration, Instant}};
use dimple_core::{collection::Collection, model::{Artist, Entity, KnownId, Recording, RecordingSource, Release, ReleaseGroup}};
use dimple_core::model::Entities;
use rayon::iter::{ParallelBridge, ParallelIterator};
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag}, probe::Hint};
use walkdir::{WalkDir, DirEntry};

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

    // TODO DRY
    fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
        match (of_type, related_to) {
            (Entities::Artist(_), None) => {
                let files = self.files.lock().unwrap().clone();
                let artists: Vec<Artist> = files.values().map(Into::into).collect();
                let models: Vec<Entities> = artists.iter().map(Artist::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::Release(_), None) => {
                let files = self.files.lock().unwrap().clone();
                let releases: Vec<Release> = files.values().map(Into::into).collect();
                let models: Vec<Entities> = releases.iter().map(Release::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::Recording(_), None) => {
                let files = self.files.lock().unwrap().clone();
                let recordings: Vec<Recording> = files.values().map(Into::into).collect();
                let models: Vec<Entities> = recordings.iter().map(Recording::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::ReleaseGroup(_), Some(Entities::Artist(artist))) => {
                let files = self.files.lock().unwrap().clone();
                let release_groups: Vec<ReleaseGroup> = files.values()
                    .filter(|r| {
                        let ra: Artist = (*r).into();
                        !ra.source_ids.is_disjoint(&artist.source_ids)
                    })
                    .map(Into::into)
                    .collect();
                let models: Vec<Entities> = release_groups.iter().map(ReleaseGroup::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::Release(_), Some(Entities::Artist(artist))) => {
                let files = self.files.lock().unwrap().clone();
                let releases: Vec<Release> = files.values()
                    .filter(|r| {
                        let ra: Artist = (*r).into();
                        !ra.source_ids.is_disjoint(&artist.source_ids)
                    })
                    .map(Into::into)
                    .collect();
                let models: Vec<Entities> = releases.iter().map(Release::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::Release(_), Some(Entities::ReleaseGroup(release_group))) => {
                let files = self.files.lock().unwrap().clone();
                let releases: Vec<Release> = files.values()
                    .filter(|file| {
                        let file_release_group: ReleaseGroup = (*file).into();
                        !file_release_group.source_ids.is_disjoint(&release_group.source_ids)
                    })
                    .map(Into::into)
                    .collect();
                let models: Vec<Entities> = releases.iter().map(Release::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::Recording(_), Some(Entities::Release(release))) => {
                let files = self.files.lock().unwrap().clone();
                let recordings: Vec<Recording> = files.values()
                    .filter(|r| {
                        let ra: Release = (*r).into();
                        !ra.source_ids.is_disjoint(&release.source_ids)
                    })
                    .map(Into::into)
                    .collect();
                let models: Vec<Entities> = recordings.iter().map(Recording::entity).collect();
                Box::new(models.into_iter())
            }
            (Entities::RecordingSource(_), Some(Entities::Recording(recording))) => {
                let files = self.files.lock().unwrap().clone();
                let sources: Vec<RecordingSource> = files.values()
                    .filter(|r| {
                        let ra: Recording = (*r).into();
                        !ra.source_ids.is_disjoint(&recording.source_ids)
                    })
                    .map(Into::into)
                    .collect();
                let models: Vec<Entities> = sources.iter().map(RecordingSource::entity).collect();
                Box::new(models.into_iter())
            }
            _ => {
                Box::new(vec![].into_iter())
            }
        }
    }

    fn stream(&self, _entity: &Entities) -> Option<Box<dyn Iterator<Item = u8>>> {
        let files = self.files.lock().unwrap().clone();
        let file = files.values()
            .find(|f| {
                let ra: RecordingSource = (*f).into();
                !ra.source_ids.is_disjoint(&_entity.source_ids())
            })?;
        log::info!("found {}", &file.path);
        Some(Box::new(std::fs::read(file.path.clone()).ok()?.into_iter()))
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

impl From<&FileDetails> for ReleaseGroup {
    fn from(value: &FileDetails) -> Self {
        Self {
            title: value.get_tag_value(StandardTagKey::Album),
            source_ids: std::iter::once(value.path.clone()).collect(),
            known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzReleaseGroupId) {
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
        let path: &Path = Path::new(&value.path);
        let extension: Option<String> = path.extension().map(|e| e.to_string_lossy().to_uppercase());
        Self {
            source_ids: std::iter::once(value.path.clone()).collect(),
            known_ids: match value.get_tag_value(StandardTagKey::MusicBrainzRecordingId) {
                Some(mbid) => std::iter::once(KnownId::MusicBrainzId(mbid)).collect(),
                _ => HashSet::default(),
            },
            extension,
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
