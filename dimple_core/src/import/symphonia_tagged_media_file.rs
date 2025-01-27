use std::{collections::{HashMap, HashSet}, fs::File, path::Path};

use lazy_static::lazy_static;
use playback_rs::Hint;
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Visual}};

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, TrackMetadata}, model::{Artist, Genre, Link, Release, Track}};

/// https://picard-docs.musicbrainz.org/en/variables/tags_basic.html
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
/// Stuff under Various Artists is great test material for Release matching.
/// WILLOW GROW is great
/// https://github.com/beetbox/beets/blob/master/beets/importer.py
/// https://github.com/beetbox/beets/blob/master/beets/importer.py#L463
/// https://picard-docs.musicbrainz.org/downloads/MusicBrainz_Picard_Tag_Map.html
/// https://github.com/pdeljanov/Symphonia/blob/master/symphonia-metadata/src/id3v2/frames.rs#L23
/// VEDMA by enjoii w. and brothel. I think those are parsed wrong.
/// A Sphere of Influence dupes
/// A Tulip Rose From The Bone Dry Dust dupes and no artists
/// Everything in Genre (17) is pretty fucked
/// "Bohren & Der Club of Gore" got split into ("Bohren", "Der Club of Gore")
#[derive(Clone, Debug)]
pub struct SymphoniaTaggedMediaFile {
    pub path: String,
    pub tags: Vec<Tag>,
    pub visuals: Vec<Visual>,
    pub length_ms: Option<u64>,
}

impl SymphoniaTaggedMediaFile {
    pub fn new(path: &Path) -> Result<SymphoniaTaggedMediaFile, anyhow::Error> {
        let media_source = File::open(&path).unwrap();
        let media_source_stream =
            MediaSourceStream::new(Box::new(media_source), Default::default());

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            hint.with_extension(extension.to_str().unwrap());
        }
        
        let mut probed = symphonia::default::get_probe()
            .format(&hint, media_source_stream, &fmt_opts, &meta_opts)?;

        let mut format = probed.format;

        let mut tags: Vec<Tag> = vec![];
        let mut visuals: Vec<Visual> = vec![];

        // Collect tags and visuals from the container.
        if let Some(metadata) = probed.metadata.get() {
            if let Some(metadata) = metadata.current() {
                tags.extend(metadata.tags().to_owned());
                visuals.extend(metadata.visuals().to_owned());
            }
        }

        // Collect tags and visuals from the format.
        if let Some(metadata) = format.metadata().current() {
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

        workaround_mixcase_std_keys(&mut tags);

        let media_file = SymphoniaTaggedMediaFile {
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

    pub fn track_metadata(&self) -> TrackMetadata {
        TrackMetadata {
            track: self.track(),
            artists: self.track_artists().into_iter().map(|artist| ArtistMetadata { artist, ..Default::default() }).collect(),
            genres: self.track_genres(),
            links: self.track_links(),
            release: Some(self.release_metadata()),
            ..Default::default()
        }   
    }

    pub fn release_metadata(&self) -> ReleaseMetadata {
        ReleaseMetadata {
            // artists: self.track_artists(),
            genres: self.track_genres(),
            links: self.track_links(),
            // release: self.release(),
            // track: self.track(),
            ..Default::default()
        }        
    }

    pub fn track(&self) -> Track {
        Track {
            key: None,
            title: self.tag(StandardTagKey::TrackTitle),
            disambiguation: None,
            summary: None,
            save: false,
            download: false,
    
            release_key: None,
            position: self.tag(StandardTagKey::TrackNumber)
                .and_then(|s| parse_n_of_m_tag(&s).0),
            length_ms: self.length_ms,        
            lyrics: self.tag(StandardTagKey::Lyrics),
            // TODO supported by some formats, find tags, Symphonia may have
            // support in v0.6.
            synchronized_lyrics: None,

            discogs_id: None,
            lastfm_id: None,
            musicbrainz_id: self.tag(StandardTagKey::MusicBrainzTrackId).or_else(|| self.tag(StandardTagKey::MusicBrainzReleaseTrackId)),
            spotify_id: None,
            wikidata_id: None,

            media_format: self.tag(StandardTagKey::MediaFormat),
            media_position: self.tag(StandardTagKey::DiscNumber)
                .and_then(|s| parse_n_of_m_tag(&s).0),
            media_title: self.tag(StandardTagKey::DiscSubtitle),
            media_track_count: self.tag(StandardTagKey::TrackTotal)
                .and_then(|s| parse_n_of_m_tag(&s).0)
                .or_else(|| self.tag(StandardTagKey::TrackNumber)
                    .and_then(|s| parse_n_of_m_tag(&s).1)),
        }
    }

    pub fn release(&self) -> Release {
        Release {
            key: None,
            title: self.tag(StandardTagKey::Album),
            disambiguation: None,
            summary: None,
            save: false,
            download: false,
    
            discogs_id: None,
            lastfm_id: None,
            musicbrainz_id: self.tag(StandardTagKey::MusicBrainzAlbumId),
            spotify_id: None,
            wikidata_id: None,

            barcode: self.tag(StandardTagKey::IdentBarcode),
            country: self.tag(StandardTagKey::ReleaseCountry),
            date: self.tag(StandardTagKey::Date),
            packaging: self.tag(StandardTagKey::MediaFormat),
            quality: None,
            status: self.tag(StandardTagKey::MusicBrainzReleaseStatus),
            release_group_type: self.tag(StandardTagKey::MusicBrainzReleaseType),
        }
    }

    pub fn track_artists(&self) -> Vec<Artist> {
        self.tags(StandardTagKey::Artist).iter()
            .flat_map(|s| parse_artist_tag(s))
            .map(|s| Artist {
                name: Some(s.to_string()),
                ..Default::default()
            })
            // .collect::<HashSet<_>>()
            // .into_iter()
            .collect()
    }

    pub fn track_genres(&self) -> Vec<Genre> {
        self.tags(StandardTagKey::Genre).iter()
            .flat_map(|s| parse_genre_tag(s))
            .map(|s| Genre {
                name: Some(s.to_string()),
                ..Default::default()
            })
            .collect()
    }

    pub fn release_artists(&self) -> Vec<Artist> {
        self.tags(StandardTagKey::AlbumArtist).iter()
            .flat_map(|s| parse_artist_tag(s))
            .map(|s| Artist {
                name: Some(s.to_string()),
                ..Default::default()
            })
            .collect()
    }

    pub fn release_genres(&self) -> Vec<Genre> {
        self.tags(StandardTagKey::Genre).iter()
            .flat_map(|s| parse_genre_tag(s))
            .map(|s| Genre {
                name: Some(s.to_string()),
                ..Default::default()
            })
            .collect()
    }

    pub fn release_links(&self) -> Vec<Link> {
        vec![]
    }

    pub fn track_links(&self) -> Vec<Link> {
        self.tag(StandardTagKey::Url).iter()
            .chain(self.tag(StandardTagKey::UrlArtist).iter())
            .chain(self.tag(StandardTagKey::UrlCopyright).iter())
            .chain(self.tag(StandardTagKey::UrlInternetRadio).iter())
            .chain(self.tag(StandardTagKey::UrlLabel).iter())
            .chain(self.tag(StandardTagKey::UrlOfficial).iter())
            .chain(self.tag(StandardTagKey::UrlPayment).iter())
            .chain(self.tag(StandardTagKey::UrlPodcast).iter())
            .chain(self.tag(StandardTagKey::UrlPurchase).iter())
            .chain(self.tag(StandardTagKey::UrlSource).iter())
            .map(|s| Link { url: s.to_string(), ..Default::default() })
            .collect()
    }
}

pub fn parse_n_of_m_tag(value: &str) -> (Option<u32>, Option<u32>) {
    let mut parts = value.splitn(2, "/").map(|val| val.trim().parse::<u32>().ok());
    (parts.next().flatten(), parts.next().flatten())
}

/// Split artist string handling various separators
/// TODO need to collect examples of the ones currently failing and add them
/// as tests.
fn parse_artist_tag(artist_str: &str) -> Vec<String> {
    let mut artists = Vec::new();
    
    // Common separators in music tags
    for part in artist_str.split(&['/', ',', ';', '&', '+'][..]) {
        let part = part.trim();
        if !part.is_empty() {
            // Handle "feat." and "featuring" specially
            if let Some(feat_pos) = part.to_lowercase().find("feat.") {
                let (artist, featuring) = part.split_at(feat_pos);
                if !artist.trim().is_empty() {
                    artists.push(artist.trim().to_string());
                }
                let feat_artist = featuring.replacen("feat.", "", 1);
                if !feat_artist.trim().is_empty() {
                    artists.push(feat_artist.trim().to_string());
                }
            } 
            else if let Some(feat_pos) = part.to_lowercase().find("featuring") {
                let (artist, featuring) = part.split_at(feat_pos);
                if !artist.trim().is_empty() {
                    artists.push(artist.trim().to_string());
                }
                let feat_artist = featuring.replacen("featuring", "", 1);
                if !feat_artist.trim().is_empty() {
                    artists.push(feat_artist.trim().to_string());
                }
            } 
            else {
                artists.push(part.to_string());
            }
        }
    }
    artists
}

/// Split genre string handling various separators
fn parse_genre_tag(genre_str: &str) -> Vec<&str> {
    genre_str.split(&['/', ',', ';'][..])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Remaps a number of standard keys in Symphonia using case insensitive
/// lookup. This seems to be a bug in 0.5.4, and is maybe fixed in 0.6-dev.
/// The tags are generated by Picard, e.g. 
/// Tag {
///   std_key: None,
///   key: "TXXX:MusicBrainz Album Id",
///   value: String(
///     "b2d5b1ab-f14b-4861-a4fa-bf0430dcfd04",
///   ),
/// },
/// https://github.com/pdeljanov/Symphonia/blob/64171c336be5519692c8149968ae2fe1fb7ef8e5/symphonia-metadata/src/id3v2/frames.rs#L686
/// https://github.com/pdeljanov/Symphonia/blob/64171c336be5519692c8149968ae2fe1fb7ef8e5/symphonia-metadata/src/id3v2/frames.rs#L23
fn workaround_mixcase_std_keys(tags: &mut[Tag]) {
    for tag in tags {
        if tag.std_key.is_none() {
            tag.std_key = TXXX_FRAME_STD_KEYS.get(&tag.key.to_uppercase()).cloned();
        }
    }
}

lazy_static! {
    static ref TXXX_FRAME_STD_KEYS: HashMap<String, StandardTagKey> = {
        let mut m = HashMap::new();
        m.insert("TXXX:ACOUSTID FINGERPRINT".to_string(), StandardTagKey::AcoustidFingerprint);
        m.insert("TXXX:ACOUSTID ID".to_string(), StandardTagKey::AcoustidId);
        m.insert("TXXX:BARCODE".to_string(), StandardTagKey::IdentBarcode);
        m.insert("TXXX:CATALOGNUMBER".to_string(), StandardTagKey::IdentCatalogNumber);
        m.insert("TXXX:LICENSE".to_string(), StandardTagKey::License);
        m.insert("TXXX:MUSICBRAINZ ALBUM ARTIST ID".to_string(), StandardTagKey::MusicBrainzAlbumArtistId);
        m.insert("TXXX:MUSICBRAINZ ALBUM ID".to_string(), StandardTagKey::MusicBrainzAlbumId);
        m.insert("TXXX:MUSICBRAINZ ARTIST ID".to_string(), StandardTagKey::MusicBrainzArtistId);
        m.insert("TXXX:MUSICBRAINZ RELEASE GROUP ID".to_string(), StandardTagKey::MusicBrainzReleaseGroupId);
        m.insert("TXXX:MUSICBRAINZ WORK ID".to_string(), StandardTagKey::MusicBrainzWorkId);
        m.insert("TXXX:REPLAYGAIN_ALBUM_GAIN".to_string(), StandardTagKey::ReplayGainAlbumGain);
        m.insert("TXXX:REPLAYGAIN_ALBUM_PEAK".to_string(), StandardTagKey::ReplayGainAlbumPeak);
        m.insert("TXXX:REPLAYGAIN_TRACK_GAIN".to_string(), StandardTagKey::ReplayGainTrackGain);
        m.insert("TXXX:REPLAYGAIN_TRACK_PEAK".to_string(), StandardTagKey::ReplayGainTrackPeak);
        m.insert("TXXX:SCRIPT".to_string(), StandardTagKey::Script);

        m.insert("TXXX:ASIN".to_string(), StandardTagKey::IdentAsin);
        m.insert("TXXX:ARTISTS".to_string(), StandardTagKey::Artist);
        m.insert("TXXX:MUSICBRAINZ ALBUM TYPE".to_string(), StandardTagKey::MusicBrainzReleaseType);
        m.insert("TXXX:MUSICBRAINZ ALBUM STATUS".to_string(), StandardTagKey::MusicBrainzReleaseStatus);
        m.insert("TXXX:MUSICBRAINZ ALBUM RELEASE COUNTRY".to_string(), StandardTagKey::ReleaseCountry);
        m.insert("TXXX:MUSICBRAINZ RELEASE TRACK ID".to_string(), StandardTagKey::MusicBrainzReleaseTrackId);
        m
    };
}

mod test {
    // #[test]
    // fn parse_n_of_m_tag() {
    //     assert!(parse_n_of_m_tag("") == (None, None));
    //     assert!(crate::import::parse_n_of_m_tag("3/12") == (Some(3), Some(12)));
    //     assert!(crate::import::parse_n_of_m_tag("1") == (Some(1), None));
    //     assert!(crate::import::parse_n_of_m_tag("1/") == (Some(1), None));
    //     assert!(crate::import::parse_n_of_m_tag("/13") == (None, Some(13)));
    //     assert!(crate::import::parse_n_of_m_tag("/") == (None, None));
    //     assert!(crate::import::parse_n_of_m_tag(" 3 /   12 ") == (Some(3), Some(12)));
    //     assert!(crate::import::parse_n_of_m_tag("03 /12 ") == (Some(3), Some(12)));
    // }
}
