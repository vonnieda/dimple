use std::{collections::{HashMap, HashSet}, path::Path};

use crate::{library::Library, merge::CrdtRules, model::{Artist, Blob, Genre, MediaFile, ModelBasics as _, Release, Track, TrackSource}};

use std::fs::File;

use anyhow::anyhow;
use lofty::{file::{AudioFile, TaggedFile, TaggedFileExt}, tag::ItemKey};
use symphonia::core::{errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Value, Visual}, probe::Hint};

pub mod spotify;

use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

// https://picard-docs.musicbrainz.org/en/variables/tags_basic.html
// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

pub fn import(library: &Library, path: &str) {
    let force = true;
    
    log::info!("Importing {}.", path);

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    files.par_iter().for_each(|file| {
        let _ = import_single_file(&library, Path::new(&file.path), force);
    });
}

fn import_single_file(library: &Library, path: &Path, force: bool) -> Result<TrackSource, anyhow::Error> {
    if !path.is_file() {
        return Err(anyhow::anyhow!("Path must be a file: {:?}", path));
    }
    
    let tags = TaggedMediaFile::new(path)?;
    
    let mut media_file = library.find_media_file_by_file_path(path.to_str().unwrap())
        .unwrap_or_default();
    media_file.file_path = path.to_str().unwrap().to_string();
    media_file.last_imported = Utc::now();
    media_file.last_modified = path.metadata()?.modified()?.into();
    let media_file = media_file.save(library);
    
    let mut track_source: TrackSource = library.find("SELECT * FROM TrackSource 
        WHERE media_file_key = ?", (&media_file.key,)).unwrap_or_default();
    
    let track = track_source.track_key
        .and_then(|track_key| Track::get(library, &track_key))
        .unwrap_or_else(|| <TaggedMediaFile as Into<Track>>::into(tags).save(library));

    track_source.track_key = track.key.clone();
    track_source.media_file_key = media_file.key.clone();
    let track_source = track_source.save(library);

    log::info!("Imported {:?} {:?}", path, track_source);

    Ok(track_source)
}

impl From<TaggedMediaFile> for Track {
    fn from(input: TaggedMediaFile) -> Self {
        Self {
            key: None,
            title: input.tag(StandardTagKey::TrackTitle),
            disambiguation: input.tag(StandardTagKey::Comment),
            summary: None,
            save: false,
            download: false,
    
            release_key: None,
            position: input.tag(StandardTagKey::TrackNumber)
                .and_then(|s| parse_n_of_m_tag(&s).0),
            length_ms: input.length_ms,        
            lyrics: input.tag(StandardTagKey::Lyrics),
            // TODO supported by some formats, find tags
            synchronized_lyrics: None,

            discogs_id: None,
            lastfm_id: None,
            musicbrainz_id: input.tag(StandardTagKey::MusicBrainzTrackId),
            spotify_id: None,
            wikidata_id: None,

            media_format: input.tag(StandardTagKey::MediaFormat),
            media_position: input.tag(StandardTagKey::DiscNumber)
                .and_then(|s| parse_n_of_m_tag(&s).0),
            media_title: input.tag(StandardTagKey::DiscSubtitle),
            media_track_count: input.tag(StandardTagKey::TrackTotal)
                .and_then(|s| parse_n_of_m_tag(&s).0)
                .or_else(|| input.tag(StandardTagKey::TrackNumber)
                    .and_then(|s| parse_n_of_m_tag(&s).1)),

            ..Default::default()
        }
    }
}

impl From<TaggedMediaFile> for Release {
    fn from(input: TaggedMediaFile) -> Self {
        Self {
            key: None,
            title: input.tag(StandardTagKey::TrackTitle),
            disambiguation: input.tag(StandardTagKey::Comment),
            summary: None,
            save: false,
            download: false,
    
            discogs_id: None,
            lastfm_id: None,
            musicbrainz_id: input.tag(StandardTagKey::MusicBrainzTrackId),
            spotify_id: None,
            wikidata_id: None,

            barcode: input.tag(StandardTagKey::IdentBarcode),
            country: input.tag(StandardTagKey::ReleaseCountry),
            date: input.tag(StandardTagKey::ReleaseDate),
            packaging: input.tag(StandardTagKey::MediaFormat),
            quality: None,
            status: input.tag(StandardTagKey::MusicBrainzReleaseStatus),
            release_group_type: input.tag(StandardTagKey::MusicBrainzReleaseType),
        }
    }
}

fn artists(input: &TaggedMediaFile) -> Vec<Artist> {
    todo!()
    // Self {
    //     key: None,
    //     name: input.tag(StandardTagKey::ar),
    //     disambiguation: input.tag(StandardTagKey::Comment),
    //     summary: None,
    //     save: false,
    //     download: false,

    //     discogs_id: None,
    //     lastfm_id: None,
    //     musicbrainz_id: input.tag(StandardTagKey::MusicBrainzTrackId),
    //     spotify_id: None,
    //     wikidata_id: None,

    //     country: input.tag(StandardTagKey::ReleaseCountry),
    // }
}

impl From<TaggedMediaFile> for Vec<Artist> {
    fn from(value: TaggedMediaFile) -> Self {
        todo!()
    }
}

impl From<TaggedMediaFile> for Genre {
    fn from(value: TaggedMediaFile) -> Self {
        todo!()
    }
}

impl From<TaggedMediaFile> for Vec<Genre> {
    fn from(value: TaggedMediaFile) -> Self {
        todo!()
    }
}
// pub fn find_track_for_media_file(&self, media_file: &MediaFile) -> Option<Track> {
//     // TODO naive, just for testing.
//     self.conn().query_row_and_then("SELECT * FROM Track
//         WHERE artist = ?1 AND album = ?2 AND title = ?3", 
//         (media_file.artist.clone(), media_file.album.clone(), media_file.title.clone()), |row| Ok(Track::from_row(row)))
//         .optional().unwrap()
// }

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
    pub fn new(path: &Path) -> Result<TaggedMediaFile, Error> {
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

    /// Returns the first tag with the specified key.
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

    /// Returns all distinct tags with the specified key.
    pub fn tags(&self, key: StandardTagKey) -> Vec<String> {
        self.tags.iter().filter_map(|t| {
            if let Some(std_key) = t.std_key {
                if std_key == key {
                    return Some(t.value.to_string())
                }
            }
            None
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
    }
}

// impl From<TaggedMediaFile> for MediaFile {
//     fn from(input: TaggedMediaFile) -> Self {
//         MediaFile {
//             key: None,
//             file_path: input.path.to_string(),
//             artist: input.tag(StandardTagKey::Artist),
//             album: input.tag(StandardTagKey::Album),
//             title: input.tag(StandardTagKey::TrackTitle),
//             genre: input.tag(StandardTagKey::Genre),
    
//             length_ms: input.length_ms,
    
//             musicbrainz_album_artist_id: input.tag(StandardTagKey::MusicBrainzAlbumArtistId),
//             musicbrainz_album_id: input.tag(StandardTagKey::MusicBrainzAlbumId),
//             musicbrainz_artist_id: input.tag(StandardTagKey::MusicBrainzArtistId),
//             musicbrainz_genre_id: input.tag(StandardTagKey::MusicBrainzGenreId),
//             musicbrainz_recording_id: input.tag(StandardTagKey::MusicBrainzRecordingId),
//             musicbrainz_release_group_id: input.tag(StandardTagKey::MusicBrainzReleaseGroupId),
//             musicbrainz_release_track_id: input.tag(StandardTagKey::MusicBrainzReleaseTrackId),
//             musicbrainz_track_id: input.tag(StandardTagKey::MusicBrainzTrackId),
    
//             lyrics: input.tag(StandardTagKey::Lyrics),
//             synced_lyrics: None,

//             // TODO hate this, this gets returned mut and then modified to fill
//             // these in. Should just be combined.
//             last_imported: Default::default(),
//             last_modified: Default::default(),
//             sha256: Default::default(),

//             disc_subtitle: input.tag(StandardTagKey::DiscSubtitle),
//             isrc: input.tag(StandardTagKey::IdentIsrc),
//             label: input.tag(StandardTagKey::Label),
//             original_date: input.tag(StandardTagKey::OriginalDate),
//             release_date: input.tag(StandardTagKey::Date),
//             disc_number: input.tag(StandardTagKey::DiscNumber)
//                 .and_then(|s| parse_n_of_m_tag(&s).0),
//             total_discs: input.tag(StandardTagKey::DiscTotal)
//                 .and_then(|s| parse_n_of_m_tag(&s).0)
//                 .or_else(|| input.tag(StandardTagKey::DiscNumber).and_then(|s| parse_n_of_m_tag(&s).1)),
//             track_number: input.tag(StandardTagKey::TrackNumber)
//                 .and_then(|s| parse_n_of_m_tag(&s).0),
//             total_tracks: input.tag(StandardTagKey::TrackTotal)
//                 .and_then(|s| parse_n_of_m_tag(&s).0)
//                 .or_else(|| input.tag(StandardTagKey::TrackNumber).and_then(|s| parse_n_of_m_tag(&s).1)),
//             // TODO lots more URLs available.
//             website: input.tag(StandardTagKey::Url),

//             original_year: None,

//             matched_track_key: None,
//         }
//     }
// }

// impl From<MediaFile> for Track {
//     fn from(media_file: MediaFile) -> Self {
//         Self {
//             artist: media_file.artist,
//             album: media_file.album,
//             title: media_file.title,
    
//             length_ms: media_file.length_ms,
    
//             lyrics: media_file.lyrics,
//             musicbrainz_id: media_file.musicbrainz_track_id,

//             disambiguation: None,
//             download: false,
//             key: None,
//             liked: false,
//             plays: 0,
//             save: false,
//             summary: None,
//             wikidata_id: None,
//             spotify_id: None,
//             synced_lyrics: media_file.synced_lyrics,

//             discogs_id: None,
//             lastfm_id: None,

//             media_position: media_file.track_number,
//         }
//     }
// }

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

// I tried Lofty but it failed on about 90 files that Symphonia doesn't.
// Trying out Lofty instead of Symphonia 
// https://github.com/pdeljanov/Symphonia/discussions/132
// let lofty = lofty::read_from_path(path)?;
// for tag in lofty.tags() {
//     let artists = tag.get_strings(&ItemKey::TrackArtist).collect::<Vec<_>>();
//     if artists.len() >= 2 {
//         dbg!(path, artists);
//     }
//     let artists = tag.get_strings(&ItemKey::AlbumArtist).collect::<Vec<_>>();
//     if artists.len() >= 2 {
//         dbg!(path, artists);
//     }
// }
// let tags = lofty.primary_tag()
//     .or_else(|| lofty.first_tag())
//     .ok_or(anyhow!("no tags"))?;
// let artists = tags.get_strings(&ItemKey::TrackArtist).collect::<Vec<_>>();
// if artists.len() == 0 || artists.len() >= 2 {
//     dbg!(path, artists);
// }
