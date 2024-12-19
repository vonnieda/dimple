use std::{collections::HashMap, path::Path};

use crate::{library::Library, merge::CrdtRules, model::{Blob, MediaFile, Track, TrackSource}};

use std::fs::File;

use symphonia::core::{errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Visual}, probe::Hint};

pub mod spotify;

use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    let force = false;
    
    log::info!("Importing {}.", path);

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    let media_files = library.list::<MediaFile>().par_iter().cloned()
        .map(|media_file| (media_file.file_path.clone(), media_file))
        .collect::<HashMap<String, MediaFile>>();
    log::info!("Loaded {} media files from database.", media_files.len());

    files.par_iter()
        .filter(|scanned_file| {
            if force {
                return true
            }
            if let Some(media_file) = media_files.get(&scanned_file.path) {
                let l = media_file.last_modified;
                let r = DateTime::<Utc>::from(scanned_file.last_modified);
                r > l
            }
            else {
                true
            }
        })        
        .filter_map(|scanned_file| {
            TaggedMediaFile::new(&scanned_file.path).ok().map(|s| (scanned_file, s))
        })
        .map(|(scanned_file, tagged_media_file)| {
            let file_path = scanned_file.path.clone();

            let blob = Blob::read(&file_path);
            let blob = library.find_blob_by_sha256(&blob.sha256)
                .or_else(|| Some(library.save(&blob)))
                .unwrap();
    
            (scanned_file, tagged_media_file, blob)
        })
        .for_each(|(scanned_file, tagged_media_file, blob)| {
            let file_path = scanned_file.path.clone();
            log::info!("Importing {}.", file_path);

            let old_media_file = media_files.get(&file_path).cloned().unwrap_or_default();
            let mut new_media_file: MediaFile = tagged_media_file.clone().into();
            new_media_file.last_modified = scanned_file.last_modified.into();
            new_media_file.last_imported = Utc::now();
            new_media_file.sha256 = blob.sha256;
            let media_file = CrdtRules::merge(old_media_file, new_media_file);
            let media_file = library.save(&media_file);

            let old_track = library.find_track_for_media_file(&media_file).unwrap_or_default();
            let new_track = media_file.into();
            let track = CrdtRules::merge(old_track, new_track);
            let track = library.save(&track);

            let _track_source = library.save(&TrackSource {
                key: None,
                blob_key: blob.key.unwrap(),
                track_key: track.key.unwrap(),
            });
        });

    // log::info!("Imported {} media files.", file_blobs.len());
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
            log::debug!("{:?} {:?}", &path, e.to_string());
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

            // TODO hate this, this gets returned mut and then modified to fill
            // these in. Should just be combined.
            last_imported: Default::default(),
            last_modified: Default::default(),
            sha256: Default::default(),

            disc_subtitle: input.tag(StandardTagKey::DiscSubtitle),
            isrc: input.tag(StandardTagKey::IdentIsrc),
            label: input.tag(StandardTagKey::Label),
            original_date: input.tag(StandardTagKey::OriginalDate),
            release_date: input.tag(StandardTagKey::Date),
            disc_number: input.tag(StandardTagKey::DiscNumber)
                .and_then(|s| parse_n_of_m_tag(&s).0),
            total_discs: input.tag(StandardTagKey::DiscTotal)
                .and_then(|s| parse_n_of_m_tag(&s).0)
                .or_else(|| input.tag(StandardTagKey::DiscNumber).and_then(|s| parse_n_of_m_tag(&s).1)),
            track_number: input.tag(StandardTagKey::TrackNumber)
                .and_then(|s| parse_n_of_m_tag(&s).0),
            total_tracks: input.tag(StandardTagKey::TrackTotal)
                .and_then(|s| parse_n_of_m_tag(&s).0)
                .or_else(|| input.tag(StandardTagKey::TrackNumber).and_then(|s| parse_n_of_m_tag(&s).1)),
            // TODO lots more URLs available.
            website: input.tag(StandardTagKey::Url),

            original_year: None,
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

            discogs_id: None,
            lastfm_id: None,

            media_position: media_file.track_number,
        }
    }
}

pub fn parse_n_of_m_tag(value: &str) -> (Option<u32>, Option<u32>) {
    let mut parts = value.splitn(2, "/").map(|val| val.trim().parse::<u32>().ok());
    (parts.next().flatten(), parts.next().flatten())
}

mod test {
    #[test]
    fn parse_n_of_m_tag() {
        assert!(crate::import::parse_n_of_m_tag("") == (None, None));
        assert!(crate::import::parse_n_of_m_tag("3/12") == (Some(3), Some(12)));
        assert!(crate::import::parse_n_of_m_tag("1") == (Some(1), None));
        assert!(crate::import::parse_n_of_m_tag("1/") == (Some(1), None));
        assert!(crate::import::parse_n_of_m_tag("/13") == (None, Some(13)));
        assert!(crate::import::parse_n_of_m_tag("/") == (None, None));
        assert!(crate::import::parse_n_of_m_tag(" 3 /   12 ") == (Some(3), Some(12)));
        assert!(crate::import::parse_n_of_m_tag("03 /12 ") == (Some(3), Some(12)));
    }
}

