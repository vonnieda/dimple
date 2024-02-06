
use std::{collections::{HashMap, HashSet}, error::Error, fs::File, sync::{Arc, Mutex}, time::{Duration, Instant}};
use dimple_core::{collection::Collection, model::Artist};
use dimple_core::model::Model;
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey}, probe::Hint};
use walkdir::{WalkDir, DirEntry};

pub struct FileLibrary {
    paths: Vec<String>,
    files: Arc<Mutex<HashMap<String, FileDetails>>>,
}

#[derive(Clone, Debug, Default)]
struct FileDetails {
    path: String,
    artist: Option<String>,
    album: Option<String>,
    title: Option<String>,
    musicbrainz_track_id: Option<String>,
    musicbrainz_recording_id: Option<String>,
    musicbrainz_release_id: Option<String>,
    musicbrainz_release_group_id: Option<String>,
    musicbrainz_artist_id: Option<String>,
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
            let file_details: Vec<_> = WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().is_some())
                .map(|e| (e.clone(), Self::read(&e)))
                .filter_map(|(_e, tag)| tag.ok())
                .collect();
            log::info!("Scanned {} files in {}ms", file_details.len(), now.elapsed().as_millis());

            let now = Instant::now();
            if let Ok(mut files) = files.lock() {
                for file in file_details {
                    files.insert(file.path.clone(), file);
                }
            }
            log::info!("Inserted files in {}ms", now.elapsed().as_millis());

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
        let probed = symphonia::default::get_probe()
            .format(&hint, media_source_stream, &fmt_opts, &meta_opts)?;

        // Get the instantiated format reader.
        let mut format = probed.format;

        let metadata = format.metadata();

        let mut details = FileDetails {
            path: path.clone(),
            ..Default::default()
        };

        if let Some(metadata) = metadata.current() {
            let tags = metadata.tags();
            // https://picard-docs.musicbrainz.org/en/variables/tags_basic.html
            for tag in tags {
                if let Some(StandardTagKey::TrackTitle) = tag.std_key {
                    details.title = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::Album) = tag.std_key {
                    details.album = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::Artist) = tag.std_key {
                    details.artist = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::MusicBrainzAlbumId) = tag.std_key {
                    details.musicbrainz_release_id = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::MusicBrainzArtistId) = tag.std_key {
                    details.musicbrainz_artist_id = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::MusicBrainzRecordingId) = tag.std_key {
                    details.musicbrainz_recording_id = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::MusicBrainzReleaseGroupId) = tag.std_key {
                    details.musicbrainz_release_group_id = Some(tag.value.to_string());
                }
                else if let Some(StandardTagKey::MusicBrainzTrackId) = tag.std_key {
                    details.musicbrainz_track_id = Some(tag.value.to_string());
                }
            }
        }

        Ok(details)
    }
}

impl Collection for FileLibrary {
    fn name(&self) -> String {
        format!("FolderLibrary({:?})", self.paths)
    }

    fn list(&self, _entity: &Model) -> Box<dyn Iterator<Item = Model>> {
        match _entity {
            Model::Artist(_) => {
                if let Ok(files) = self.files.lock() {
                    // Collect into a HashSet to deduplicate.
                    let artist_ids: HashSet<String> = files.values()
                        .filter_map(|f| f.musicbrainz_artist_id.clone())
                        .collect();

                    let results: Vec<_> = artist_ids.iter()
                        .map(|id| Model::Artist(Artist::from_id(id)))
                        .collect();

                    log::info!("list {} artists", results.len());
                    return Box::new(results.into_iter());
                }
                Box::new(vec![].into_iter())
            }
            _ => {
                Box::new(vec![].into_iter())
            }
        }
    }
}

