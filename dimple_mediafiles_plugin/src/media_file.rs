use std::{fs::File, path::PathBuf};

use dimple_core::{db::Db, model::{Artist, KnownId, Medium, Recording, RecordingSource, Release, ReleaseGroup, Track}};
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Visual}, probe::Hint};

#[derive(Clone, Debug)]
pub struct MediaFile {
    path: PathBuf,
    tags: Vec<Tag>,
    visuals: Vec<Visual>,

    // pub key: Option<String>,

    // pub url: String,
    // pub created_at: Instant,
    // pub modified_at: Instant,
    // pub artist: Option<String>,
    // pub album: Option<String>,
    // pub album_artist: Option<String>,
    // pub title: Option<String>,
    // pub genre: Option<String>,

    // pub recording_mbid: Option<String>,
    // pub release_track_mbid: Option<String>,
    // pub album_mbid: Option<String>,
    // pub artist_mbid: Option<String>,
    // pub album_artist_mbid: Option<String>,
    // pub mb_album_type: Option<String>,
    // pub mb_album_comment: Option<String>,
}

impl MediaFile {
    pub fn new(path: &PathBuf) -> anyhow::Result<MediaFile> {
        let extension = path.extension().map(|f| f.to_string_lossy().to_string());

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

        let media_file = MediaFile {
            path: path.clone(),
            tags,
            visuals,
        };

        log::info!("{}", path.to_str().unwrap());
        for tag in media_file.tags.iter() {
            log::info!("  Tag: {} = {}", tag.key, tag.value);
        }
        for visual in media_file.visuals.iter() {
            log::info!("  Visual: {} {:?} {:?}", visual.media_type, visual.dimensions, visual.usage);
            for tag in visual.tags.iter() {
                log::info!("    Tag: {} = {}", tag.key, tag.value);
            }
        }

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

    // TODO It may do to make this return a list, because of musicbrainz_artistid
    // being multi-value.
    pub fn artist(&self) -> Artist {
        Artist {
            name: self.tag(StandardTagKey::Artist),
            known_ids: self.tag(StandardTagKey::MusicBrainzArtistId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .collect(),
            links: self.tag(StandardTagKey::UrlArtist).iter()
                .cloned()
                .collect(),
            ..Default::default()
        }
    }

    pub fn album_artist(&self) -> Artist {
        Artist {
            name: self.tag(StandardTagKey::AlbumArtist),
            known_ids: self.tag(StandardTagKey::MusicBrainzAlbumArtistId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .collect(),
            ..Default::default()
        }
    }

    pub fn release_group(&self) -> ReleaseGroup {
        ReleaseGroup {
            title: self.tag(StandardTagKey::Album),
            primary_type: self.tag(StandardTagKey::MusicBrainzReleaseType),
            first_release_date: self.tag(StandardTagKey::ReleaseDate),
            known_ids: self.tag(StandardTagKey::MusicBrainzReleaseGroupId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .collect(),
            ..Default::default()
        }
    }

    pub fn release(&self) -> Release {
        Release {
            title: self.tag(StandardTagKey::Album),
            status: self.tag(StandardTagKey::MusicBrainzReleaseStatus),
            country: self.tag(StandardTagKey::ReleaseCountry),
            date: self.tag(StandardTagKey::Date),
            barcode: self.tag(StandardTagKey::IdentBarcode),
            known_ids: self.tag(StandardTagKey::MusicBrainzAlbumId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .chain(self.tag(StandardTagKey::IdentBarcode).iter().map(|id| KnownId::Barcode(id.to_string())))
                .collect(),
            ..Default::default()
        }
    }

    pub fn medium(&self) -> Medium {
        Medium {
            title: self.tag(StandardTagKey::DiscSubtitle),
            disc_count: self.tag(StandardTagKey::DiscTotal).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            position: self.tag(StandardTagKey::DiscNumber).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            track_count: self.tag(StandardTagKey::TrackTotal).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            format: self.tag(StandardTagKey::MediaFormat),
            ..Default::default()
        }
    }

    pub fn track(&self) -> Track {
        Track {
            title: self.tag(StandardTagKey::DiscSubtitle),
            // TODO length
            length: Default::default(),
            number: self.tag(StandardTagKey::TrackNumber).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            position: self.tag(StandardTagKey::DiscNumber).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            known_ids: self.tag(StandardTagKey::MusicBrainzTrackId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .chain(self.tag(StandardTagKey::MusicBrainzReleaseTrackId).iter().map(|id| KnownId::MusicBrainzId(id.to_string())))
                .collect(),
            ..Default::default()
        }
    }

    pub fn recording(&self) -> Recording {
        Recording {
            title: self.tag(StandardTagKey::TrackTitle),
            disambiguation: self.tag(StandardTagKey::TrackSubtitle),
            // TODO length
            length: Default::default(),
            annotation: self.tag(StandardTagKey::Comment),
            isrc: self.tag(StandardTagKey::IdentIsrc),
            known_ids: self.tag(StandardTagKey::MusicBrainzRecordingId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .chain(self.tag(StandardTagKey::IdentIsrc).iter().map(|id| KnownId::ISRC(id.to_string())))
                .chain(self.tag(StandardTagKey::IdentAsin).iter().map(|id| KnownId::ASIN(id.to_string())))
                .collect(),
            links: self.tag(StandardTagKey::Url).iter()
                .chain(self.tag(StandardTagKey::UrlOfficial).iter())
                .chain(self.tag(StandardTagKey::UrlPurchase).iter())
                .cloned()
                .collect(),
            ..Default::default()
        }
    }

    pub fn recording_source(&self) -> RecordingSource {
        RecordingSource {
            extension: self.path.extension().map(|e| e.to_string_lossy().to_lowercase()),
            source_id: format!("dmfp://{}", self.path.to_str().unwrap_or_default()),
            ..Default::default()
        }
    }

    // TODO genres


    /// Takes a MediaFile with unresolved entities and returns a new one
    /// with matching entities.
    pub fn find_matching(&self, db: &dyn Db) -> MediaFile {
        todo!()
    }
}


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
