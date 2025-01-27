use std::{collections::{HashMap, HashSet}, fs::File, path::Path, sync::Arc};

use anyhow::anyhow;
use image::DynamicImage;
use itertools::Itertools;
use lofty::{file::{TaggedFile, TaggedFileExt}, picture::PictureType, tag::{Accessor, ItemKey, Tag, TagExt, TagItem, TagType}};

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, TrackMetadata}, model::{dimage::DimageKind, Artist, Dimage, Genre, Link, Release, Track}};

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
#[derive(Clone)]
pub struct LoftyTaggedMediaFile {
    pub path: String,
    pub tags: Tag,
}

impl LoftyTaggedMediaFile {
    pub fn new(path: &Path) -> Result<LoftyTaggedMediaFile, anyhow::Error> {
        let tagged_file = lofty::read_from_path(path)?;
        let tag = tagged_file.primary_tag()
            .or(tagged_file.first_tag())
            .ok_or(anyhow!("No tags found."))?;

        let media_file = LoftyTaggedMediaFile {
            path: path.to_str().unwrap().to_string(),
            tags: tag.clone(),
        };

        Ok(media_file)
    }

    pub fn images(&self) -> Vec<Dimage> {
        self.tags.pictures().iter().filter_map(|pic| {
            if let Ok(dymage) = image::load_from_memory(pic.data()) {
                let mut dimage = Dimage::new(&dymage);
                dimage.kind = match pic.pic_type() {
                    // TODO more
                    PictureType::Artist => Some(DimageKind::MusicArtistThumb),
                    _ => None,
                };
                Some(dimage)
            }
            else {
                None
            }
        })
        .collect()
    }

    pub fn track_metadata(&self) -> TrackMetadata {
        TrackMetadata {
            track: self.track(),
            artists: self.track_artists(),
            genres: self.track_genres(),
            links: self.track_links(),
            release: Some(self.release_metadata()),            
            images: self.images(),
        }   
    }

    fn track(&self) -> Track {
        Track {
            title: self.tags.title().map(Into::into),
            position: self.tags.track(),
            length_ms: self.tags.get_string(&ItemKey::Length).map(|l| u64::from_str_radix(l, 10).ok()).flatten(),
            lyrics: self.tags.get_string(&ItemKey::Lyrics).map(Into::into),
            musicbrainz_id: self.tags.get_string(&ItemKey::MusicBrainzTrackId).map(Into::into),
            media_position: self.tags.disk(),
            media_track_count: self.tags.track_total(),
            ..Default::default()
        }
    }

    fn release_metadata(&self) -> ReleaseMetadata {
        ReleaseMetadata {
            release: self.release(),
            artists: self.release_artists(),
            links: self.release_links(),
            genres: self.release_genres(),            
            images: self.images(),
            tracks: vec![],
        }
    }    

    fn release_genres(&self) -> Vec<Genre> {
        self.tags.get_string(&ItemKey::Genre).iter()
            .flat_map(|s| parse_genre_tag(s))
            .map(|s| Genre {
                name: Some(s.to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn track_genres(&self) -> Vec<Genre> {
        self.tags.get_string(&ItemKey::Genre).iter()
            .flat_map(|s| parse_genre_tag(s))
            .map(|s| Genre {
                name: Some(s.to_string()),
                ..Default::default()
            })
            .collect()        
    }

    fn release(&self) -> Release {
        Release {
            title: self.tags.album().map(Into::into),
            barcode: self.tags.get_string(&ItemKey::Barcode).map(Into::into),
            musicbrainz_id: self.tags.get_string(&ItemKey::MusicBrainzReleaseId).map(Into::into),
            ..Default::default()
        }    
    }

    fn release_artists(&self) -> Vec<ArtistMetadata> {
        let artists = self.tags.get_strings(&ItemKey::AlbumArtist)
            .zip_longest(self.tags.get_strings(&ItemKey::MusicBrainzReleaseArtistId))
            .map(|artist| {
                ArtistMetadata {
                    artist: Artist {
                        name: artist.clone().left().map(Into::into),
                        musicbrainz_id: artist.clone().right().map(Into::into),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .collect::<Vec<ArtistMetadata>>();
        if !artists.is_empty() {
            return artists
        }
        artists
    }

    fn track_artists(&self) -> Vec<ArtistMetadata> {
        let artists: Vec<_> = self.tags.get_strings(&ItemKey::TrackArtists)
            .zip_longest(self.tags.get_strings(&ItemKey::MusicBrainzArtistId))
            .map(|artist| {
                ArtistMetadata {
                    artist: Artist {
                        name: artist.clone().left().map(Into::into),
                        musicbrainz_id: artist.clone().right().map(Into::into),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .collect();
        if !artists.is_empty() {
            return artists
        }
        self.tags.get_strings(&ItemKey::TrackArtist)
            .flat_map(|a| parse_artist_tag(a))
            .zip_longest(self.tags.get_strings(&ItemKey::MusicBrainzArtistId))
            .map(|artist| {
                ArtistMetadata {
                    artist: Artist {
                        name: artist.clone().left().map(Into::into),
                        musicbrainz_id: artist.clone().right().map(Into::into),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .collect()
    }
    
    fn release_links(&self) -> Vec<Link> {
        vec![]
    }

    fn track_links(&self) -> Vec<Link> {
        self.tags.get_string(&ItemKey::PaymentUrl).iter()
            .chain(self.tags.get_string(&ItemKey::AudioFileUrl).iter())
            .chain(self.tags.get_string(&ItemKey::CopyrightUrl).iter())
            .chain(self.tags.get_string(&ItemKey::PublisherUrl).iter())
            .chain(self.tags.get_string(&ItemKey::AudioSourceUrl).iter())
            .chain(self.tags.get_string(&ItemKey::RadioStationUrl).iter())
            .chain(self.tags.get_string(&ItemKey::CommercialInformationUrl).iter())
            .map(|s| Link { url: s.to_string(), ..Default::default() })
            .collect()
    }
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
