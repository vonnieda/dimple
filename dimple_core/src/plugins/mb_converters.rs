use musicbrainz_rs::entity::relations::RelationContent;

use crate::{
    librarian::{ArtistMetadata, ReleaseGroupMetadata, ReleaseMetadata, TrackMetadata},
    model::{Artist, Link, Release, ReleaseGroup, ReleaseGroupPrimaryType, ReleaseGroupSecondaryType, Track},
};

// Note that in the converters below ..Default should never be used. If a Default
// is temporarily needed it can be specified on the field itself, but not
// the entire struct. This is to help avoid skipping fields when new ones
// are added.

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub struct ArtistConverter(musicbrainz_rs::entity::artist::Artist);

impl From<musicbrainz_rs::entity::artist::Artist> for ArtistConverter {
    fn from(value: musicbrainz_rs::entity::artist::Artist) -> Self {
        ArtistConverter(value)
    }
}

impl From<ArtistConverter> for ArtistMetadata {
    fn from(value: ArtistConverter) -> Self {
        Self {
            artist: Artist {
                country: value.0.country,
                disambiguation: none_if_empty(value.0.disambiguation),
                id: None,
                musicbrainz_id: Some(value.0.id.clone()),
                name: none_if_empty(value.0.name),
                summary: None,
                ..Default::default()
            },
            // releases: value.0.releases.iter().flatten()
            //     .map(|r| ReleaseMetadata::from(ReleaseConverter::from(r.to_owned())))
            //     .collect(),
            genres: value.0.genres.iter().flatten()
                .map(|f| crate::model::Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(
                    std::iter::once(value.0.id.clone())
                        .map(|mbid| format!("https://musicbrainz.org/artist/{mbid}")),
                )
                .map(|s| Link {
                    id: None,
                    name: None,
                    url: s,
                })
                .collect(),
            images: vec![],
        }
    }
}

pub struct ReleaseConverter(musicbrainz_rs::entity::release::Release);

impl From<musicbrainz_rs::entity::release::Release> for ReleaseConverter {
    fn from(value: musicbrainz_rs::entity::release::Release) -> Self {
        ReleaseConverter(value)
    }
}

impl From<ReleaseConverter> for ReleaseMetadata {
    fn from(value: ReleaseConverter) -> Self {
        Self {
            release: Release {
                barcode: value.0.barcode,
                country: value.0.country,
                date: value.0.date.map(|f| f.to_string()),
                disambiguation: value.0.disambiguation,
                id: None,
                musicbrainz_id: Some(value.0.id.clone()),
                title: none_if_empty(value.0.title),
                packaging: value.0.packaging.map(|f| format!("{f:?}")),
                status: value.0.status.map(|f| format!("{f:?}")),
                quality: value.0.quality.map(|f| format!("{f:?}")),
                summary: None,
                // media: value.0.media.iter().flatten()
                //     .map(|f| Medium::from(MediumConverter::from(f.to_owned())))
                //     .collect(),
                ..Default::default()
            },
            artists: value.0.artist_credit.iter().flatten()
                .map(|f| ArtistMetadata::from(ArtistConverter::from(f.artist.to_owned())))
                .collect(),
            genres: value.0.genres.iter().flatten()
                .map(|f| crate::model::Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(
                    std::iter::once(value.0.id.clone())
                        .map(|mbid| format!("https://musicbrainz.org/release/{mbid}")),
                )
                .map(|s| Link {
                    id: None,
                    name: None,
                    url: s,
                })
                .collect(),
            // TODO need to pull in the media data here, for the track fields
            // that denormalize it.
            tracks: value.0.media.iter().flatten()
                .flat_map(|media| media.tracks.iter())
                .flat_map(|tracks| tracks.iter())
                .map(|track| TrackMetadata::from(TrackConverter::from(track.to_owned())))
                .collect(),
            images: vec![],
            // release_group: value.0.release_group.map(|f| ReleaseGroupMetadata::from(ReleaseGroupConverter::from(f.to_owned()))).unwrap()
        }
    }
}

pub struct ReleaseGroupConverter(musicbrainz_rs::entity::release_group::ReleaseGroup);

impl From<musicbrainz_rs::entity::release_group::ReleaseGroup> for ReleaseGroupConverter {
    fn from(value: musicbrainz_rs::entity::release_group::ReleaseGroup) -> Self {
        ReleaseGroupConverter(value)
    }
}

impl From<musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType> for ReleaseGroupPrimaryType {
    fn from(value: musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType) -> Self {
        match value {
            musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType::Album => crate::model::ReleaseGroupPrimaryType::Album,
            musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType::Single => crate::model::ReleaseGroupPrimaryType::Single,
            musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType::Ep => crate::model::ReleaseGroupPrimaryType::EP,
            musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType::Broadcast => crate::model::ReleaseGroupPrimaryType::Broadcast,
            musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType::Other => crate::model::ReleaseGroupPrimaryType::Other,
            _ => crate::model::ReleaseGroupPrimaryType::Unrecognized,
        }
    }
}

impl From<musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType> for ReleaseGroupSecondaryType {
    fn from(value: musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType) -> Self {
        match value {
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::AudioDrama => crate::model::ReleaseGroupSecondaryType::AudioDrama,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Audiobook => crate::model::ReleaseGroupSecondaryType::Audiobook,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Compilation => crate::model::ReleaseGroupSecondaryType::Compilation,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Demo => crate::model::ReleaseGroupSecondaryType::DJMix,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::DjMix => crate::model::ReleaseGroupSecondaryType::Demo,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Interview => crate::model::ReleaseGroupSecondaryType::Interview,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Live => crate::model::ReleaseGroupSecondaryType::Live,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::MixtapeStreet => crate::model::ReleaseGroupSecondaryType::MixtapeStreet,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Remix => crate::model::ReleaseGroupSecondaryType::Remix,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Soundtrack => crate::model::ReleaseGroupSecondaryType::Soundtrack,
            musicbrainz_rs::entity::release_group::ReleaseGroupSecondaryType::Spokenword => crate::model::ReleaseGroupSecondaryType::Spokenword,
            _ => crate::model::ReleaseGroupSecondaryType::Unrecognized,
        }
    }
}

impl From<ReleaseGroupConverter> for ReleaseGroupMetadata {
    fn from(value: ReleaseGroupConverter) -> Self {
        Self {
            release_group: ReleaseGroup {
                first_release_date: value.0.first_release_date.map(|f| f.to_string()),
                disambiguation: none_if_empty(value.0.disambiguation),
                id: None,
                musicbrainz_id: Some(value.0.id.clone()),
                title: none_if_empty(value.0.title),
                primary_type: value.0.primary_type.map(Into::into),
                ..Default::default()                
            },
            secondary_types: value.0.secondary_types.iter().cloned().map(Into::into).collect::<Vec<_>>(),
            artists: value.0.artist_credit.iter().flatten()
                .map(|f| ArtistMetadata::from(ArtistConverter::from(f.artist.to_owned())))
                .collect(),
            genres: value.0.genres.iter().flatten()
                .map(|f| crate::model::Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            // releases: value.0.releases.iter().flatten()
            //     .map(|f| ReleaseMetadata::from(ReleaseConverter::from(f.to_owned())))
            //     .collect(),
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(
                    std::iter::once(value.0.id.clone())
                        .map(|mbid| format!("https://musicbrainz.org/release-group/{mbid}")),
                )
                .map(|s| Link {
                    id: None,
                    name: None,
                    url: s,
                })
                .collect(),
            images: vec![],
        }
    }
}

pub struct TrackConverter(musicbrainz_rs::entity::release::Track);

impl From<musicbrainz_rs::entity::release::Track> for TrackConverter {
    fn from(value: musicbrainz_rs::entity::release::Track) -> Self {
        TrackConverter(value)
    }
}

// pub struct Track {
//     pub recording: Option<Recording>,
//     pub title: String,
//     pub number: String,
//     pub length: Option<u32>,
//     pub position: u32,
//     pub id: String,
//     pub artist_credit: Option<Vec<ArtistCredit>>,
// }
impl From<TrackConverter> for TrackMetadata {
    fn from(value: TrackConverter) -> Self {
        Self {
            track: Track {
                id: None,
                musicbrainz_id: Some(value.0.id.clone()),
                position: Some(value.0.position),
                title: none_if_empty(value.0.title),
                disambiguation: None,
                summary: None,
                save: false,
                download: false,
                release_id: None,
                length_ms: value.0.length.map(|l| l as u64),
                // lyrics: None,
                // synchronized_lyrics: None,
                recording_id: None,
                discogs_id: None,
                lastfm_id: None,
                spotify_id: None,
                wikidata_id: None,
                media_track_count: None,
                media_position: Some(value.0.position),
                media_title: None,
                media_format: None,
            },
            artists: value.0.artist_credit.iter().flatten()
                .map(|f| ArtistMetadata::from(ArtistConverter::from(f.artist.to_owned())))
                .collect(),
            genres: value.0.recording.iter().flat_map(|r| r.genres.as_ref()).flat_map(|g| g.iter())
                .map(|f| crate::model::Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            links: value.0.recording.iter().flat_map(|r| r.relations.as_ref()).flat_map(|r| r.iter())
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(
                    std::iter::once(value.0.id.clone())
                        .map(|mbid| format!("https://musicbrainz.org/track/{mbid}")),
                )
                .map(|s| Link {
                    id: None,
                    name: None,
                    url: s,
                })
                .collect(),
            images: vec![],
        }
    }
}

pub struct GenreConverter(musicbrainz_rs::entity::genre::Genre);

impl From<musicbrainz_rs::entity::genre::Genre> for GenreConverter {
    fn from(value: musicbrainz_rs::entity::genre::Genre) -> Self {
        GenreConverter(value)
    }
}

impl From<GenreConverter> for crate::model::Genre {
    fn from(value: GenreConverter) -> Self {
        crate::model::Genre {
            disambiguation: None,
            id: None,
            musicbrainz_id: value.0.id,
            name: none_if_empty(value.0.name),
            summary: None,
            ..Default::default()
        }
    }
}
