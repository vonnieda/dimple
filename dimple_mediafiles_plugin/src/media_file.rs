use std::{fs::File, path::PathBuf};

use dimple_core::model::{Artist, Genre, KnownId, KnownIds, Medium, Recording, RecordingSource, Release, ReleaseGroup, Track};
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Visual}, probe::Hint};

#[derive(Clone, Debug)]
pub struct MediaFile {
    pub path: PathBuf,
    pub tags: Vec<Tag>,
    pub visuals: Vec<Visual>,
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

    // TODO This and a few others may need to be a list, since some of the tags
    // have multiple values.
    pub fn artist(&self) -> Artist {
        Artist {
            name: self.tag(StandardTagKey::Artist),
            known_ids: KnownIds { 
                musicbrainz_id: self.tag(StandardTagKey::MusicBrainzArtistId),
                ..Default::default()
            },
            links: self.tag(StandardTagKey::UrlArtist).iter()
                .cloned()
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
            primary_type: self.tag(StandardTagKey::MusicBrainzReleaseType),
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
            title: self.tag(StandardTagKey::TrackTitle),
            // TODO length
            length: Default::default(),
            number: self.tag(StandardTagKey::TrackNumber).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            position: self.tag(StandardTagKey::DiscNumber).and_then(|s| u32::from_str_radix(&s, 10).ok()),
            known_ids: self.tag(StandardTagKey::MusicBrainzReleaseTrackId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
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
            known_ids: self.tag(StandardTagKey::MusicBrainzTrackId).iter().map(|id| KnownId::MusicBrainzId(id.to_string()))
                .chain(self.tag(StandardTagKey::MusicBrainzRecordingId).iter().map(|id| KnownId::MusicBrainzId(id.to_string())))
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
    
    pub fn genres(&self) -> Vec<Genre> {
        if let Some(genre_name) = self.tag(StandardTagKey::Genre) {
            let genres: Vec<_> = genre_name.split(";")
                .map(|genre_name| Genre {
                    name: Some(genre_name.to_string()),
                    // TODO how to map musicbrainz id when multiple genres?
                    // known_ids: 
                    ..Default::default()
                })
                .collect();
            genres
        }
        else {
            vec![]
        }
    }
}

