use std::{collections::HashMap, path::Path};

use crate::{library::Library, merge::Crdt, model::{Blob, MediaFile, Track, TrackSource}};

use std::fs::File;

use symphonia::core::{errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Visual}, probe::Hint};

pub mod spotify;

use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    log::info!("Importing {}.", path);

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    let media_files = library.list::<MediaFile>().par_iter().cloned()
        .map(|media_file| (media_file.file_path.clone(), media_file))
        .collect::<HashMap<String, MediaFile>>();
    log::info!("Loaded {} media files from database.", media_files.len());

    let to_import = files.par_iter().filter(|file| {
        if let Some(media_file) = media_files.get(&file.path) {
            let l = media_file.last_modified;
            let r = DateTime::<Utc>::from(file.last_modified);
            r > l
        }
        else {
            true
        }
    }).collect::<Vec<_>>();
    log::info!("{} new or modified files to import.", to_import.len());

    let imports = to_import.par_iter()
        .filter_map(|file| {
            TaggedMediaFile::new(&file.path)
                .ok()
                .map(|s| (file, s))
        })
        .collect::<Vec<_>>();
    log::info!("Parsed {} files.", imports.len());

    // TODO this doesn't clean up old blobs that have changed?

    let file_blobs = imports.par_iter().map(|file| {
        let file_path = file.1.path.clone();

        let blob = Blob::read(&file_path);
        let blob = library.find_blob_by_sha256(&blob.sha256)
            .or_else(|| Some(library.save(&blob)))
            .unwrap();

        log::info!("Hashed {} -> {}.", file_path, blob.sha256);

        (file, blob)
    }).collect::<Vec<_>>();

    
    file_blobs.iter().enumerate().for_each(|(i, file_blob)| {
        let file = file_blob.0;
        let blob = file_blob.1.clone();
        let file_path = file.1.path.clone();
        log::info!("Importing {} of {}: {}.", i, imports.len(), file_path);

        let old_media_file = media_files.get(&file_blob.0.1.path).cloned().unwrap_or_default();
        // TODO refactor this to merge ScurnedFale and ScannedFile
        let mut new_media_file: MediaFile = file_blob.0.1.clone().into();
        new_media_file.last_modified = file_blob.0.0.last_modified.into();
        new_media_file.last_imported = Utc::now();
        new_media_file.sha256 = blob.sha256;
        let media_file = Crdt::crdt(old_media_file, new_media_file);
        let media_file = library.save(&media_file);

        let old_track = library.find_track_for_media_file(&media_file).unwrap_or_default();
        let new_track = media_file.into();
        let track = Crdt::crdt(old_track, new_track);
        let track = library.save(&track);

        let _track_source = library.save(&TrackSource {
            key: None,
            blob_key: blob.key.unwrap(),
            track_key: track.key.unwrap(),
        });
    });

    log::info!("Imported {} media files.", imports.len());
}

fn scan(path: &str) -> Vec<ScannedFile> {
    let files = WalkDir::new(path).into_iter()
        .filter_map(|dir_entry| dir_entry.ok())
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .filter(|dir_entry| dir_entry.file_name() != ".DS_Store")
        .map(|dir_entry| ScannedFile {
            path: dir_entry.path().to_str().unwrap().to_string(),
            last_modified: dir_entry.metadata().unwrap().modified().unwrap().into(),
            file_length: dir_entry.metadata().unwrap().len(),
        })
        .collect::<Vec<_>>();
    files
}

#[derive(Debug)]
struct ScannedFile {
    path: String,
    last_modified: DateTime<Utc>,
    file_length: u64,
}

#[derive(Clone, Debug)]
pub struct TaggedMediaFile {
    pub path: String,
    pub tags: Vec<Tag>,
    pub visuals: Vec<Visual>,
    pub length_ms: Option<u64>,
}

impl TaggedMediaFile {
    pub fn new(path: &str) -> Result<TaggedMediaFile, Error> {
        let path = Path::new(path);
        let media_source = File::open(&path).unwrap();
        let media_source_stream =
            MediaSourceStream::new(Box::new(media_source), Default::default());

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            hint.with_extension(extension.to_str().unwrap());
        }
        
        let probed = symphonia::default::get_probe()
            .format(&hint, media_source_stream, &fmt_opts, &meta_opts);

        if let Err(e) = probed {
            log::error!("{:?} {:?}", &path, e.to_string());
            return Err(e)
        }

        let mut probed = probed.unwrap();

        let mut format = probed.format;

        let mut tags: Vec<Tag> = vec![];
        let mut visuals: Vec<Visual> = vec![];

        if let Some(metadata) = probed.metadata.get() {
            if let Some(metadata) = metadata.current() {
                tags.extend(metadata.tags().to_owned());
                visuals.extend(metadata.visuals().to_owned());
            }
        }

        let metadata = format.metadata();

        if let Some(metadata) = metadata.current() {
            tags.extend(metadata.tags().to_owned());
            visuals.extend(metadata.visuals().to_owned());
        }

        let mut length_ms = None;
        if let Some(track) = format.tracks().get(0) {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    let length = time_base.calc_time(n_frames);
                    length_ms = Some((length.seconds * 1000) + ((length.frac * 1000.) as u64));
                }
            }
        }

        let media_file = TaggedMediaFile {
            path: path.to_str().unwrap().to_string(),
            tags,
            visuals,
            length_ms,
        };

        Ok(media_file)
    }

    pub fn tag(&self, key: StandardTagKey) -> Option<String> {
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

impl From<TaggedMediaFile> for MediaFile {
    fn from(input: TaggedMediaFile) -> Self {
        MediaFile {
            key: None,
            // TODO hate this
            sha256: Default::default(),
            file_path: input.path.to_string(),
            artist: input.tag(StandardTagKey::Artist),
            album: input.tag(StandardTagKey::Album),
            title: input.tag(StandardTagKey::TrackTitle),
            genre: input.tag(StandardTagKey::Genre),
    
            length_ms: input.length_ms,
    
            musicbrainz_album_artist_id: input.tag(StandardTagKey::MusicBrainzAlbumArtistId),
            musicbrainz_album_id: input.tag(StandardTagKey::MusicBrainzAlbumId),
            musicbrainz_artist_id: input.tag(StandardTagKey::MusicBrainzArtistId),
            musicbrainz_genre_id: input.tag(StandardTagKey::MusicBrainzGenreId),
            musicbrainz_recording_id: input.tag(StandardTagKey::MusicBrainzRecordingId),
            musicbrainz_release_group_id: input.tag(StandardTagKey::MusicBrainzReleaseGroupId),
            musicbrainz_release_track_id: input.tag(StandardTagKey::MusicBrainzReleaseTrackId),
            musicbrainz_track_id: input.tag(StandardTagKey::MusicBrainzTrackId),
    
            lyrics: input.tag(StandardTagKey::Lyrics),
            synced_lyrics: None,

            last_imported: Default::default(),
            last_modified: Default::default(),
        }
    }
}

impl From<MediaFile> for Track {
    fn from(media_file: MediaFile) -> Self {
        Self {
            artist: media_file.artist,
            album: media_file.album,
            title: media_file.title,
    
            length_ms: media_file.length_ms,
    
            lyrics: media_file.lyrics,
            musicbrainz_id: media_file.musicbrainz_track_id,

            disambiguation: None,
            download: false,
            key: None,
            liked: false,
            plays: 0,
            save: false,
            summary: None,
            wikidata_id: None,
            spotify_id: None,
            synced_lyrics: media_file.synced_lyrics,
        }
    }
}

