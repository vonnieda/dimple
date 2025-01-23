use musicbrainz_rs::entity::relations::RelationContent;

use crate::{
    librarian::{ArtistMetadata, ReleaseMetadata},
    model::{Artist, Link, Release},
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
                key: None,
                musicbrainz_id: Some(value.0.id.clone()),
                name: none_if_empty(value.0.name),
                summary: None,
                ..Default::default()
            },
            genres: value
                .0
                .genres
                .iter()
                .flatten()
                .map(|f| crate::model::Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            links: value
                .0
                .relations
                .iter()
                .flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(
                    std::iter::once(value.0.id.clone())
                        .map(|mbid| format!("https://musicbrainz.org/artist/{}", mbid)),
                )
                .map(|s| Link {
                    key: None,
                    name: None,
                    url: s,
                })
                .collect(),
            ..Default::default()
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
                key: None,
                musicbrainz_id: Some(value.0.id.clone()),
                title: none_if_empty(value.0.title),
                packaging: value.0.packaging.map(|f| format!("{:?}", f)),
                // primary_type: value.0.release_group.clone().and_then(|rg| rg.primary_type).and_then(|pt| Some(format!("{:?}", pt))),
                // release_group: value.0.release_group
                //     .map(|f| ReleaseGroup::from(ReleaseGroupConverter::from(f.to_owned()))).unwrap(),
                status: value.0.status.map(|f| format!("{:?}", f)),
                quality: value.0.quality.map(|f| format!("{:?}", f)),
                summary: None,
                // media: value.0.media.iter().flatten()
                //     .map(|f| Medium::from(MediumConverter::from(f.to_owned())))
                //     .collect(),
                ..Default::default()
            },
            genres: value
                .0
                .genres
                .iter()
                .flatten()
                .map(|f| crate::model::Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            links: value
                .0
                .relations
                .iter()
                .flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(
                    std::iter::once(value.0.id.clone())
                        .map(|mbid| format!("https://musicbrainz.org/release/{}", mbid)),
                )
                .map(|s| Link {
                    key: None,
                    name: None,
                    url: s,
                })
                .collect(),
            ..Default::default()
        }
    }
}

pub struct TrackConverter(musicbrainz_rs::entity::release::Track);

impl From<musicbrainz_rs::entity::release::Track> for TrackConverter {
    fn from(value: musicbrainz_rs::entity::release::Track) -> Self {
        TrackConverter(value)
    }
}

impl From<TrackConverter> for crate::model::Track {
    fn from(value: TrackConverter) -> Self {
        crate::model::Track {
            // artist_credits: value.0.recording.artist_credit.iter().flatten()
            //     .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
            //     .collect(),
            // genres: value.0.recording.genres.iter().flatten()
            //     .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
            //     .collect(),
            key: None,
            musicbrainz_id: Some(value.0.id),
            // length: value.0.length,
            // number: u32::from_str_radix(&value.0.number, 10).ok(),
            position: Some(value.0.position),
            title: none_if_empty(value.0.title),

            // links: value.0.relations.iter().flatten()
            //     .filter_map(|r| match &r.content {
            //         RelationContent::Url(u) => Some(u.resource.to_string()),
            //         _ => None,
            //     })
            //     .chain(std::iter::once(value.0.id.clone())
            //         .map(|mbid| format!("https://musicbrainz.org/release/{}", mbid)))
            //     .map(|s| Link { key: None, name: None, url: s })
            //     .collect(),
            ..Default::default()
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
            key: None,
            musicbrainz_id: value.0.id,
            //     ..Default::default()
            // },
            // links: value.0.relations.iter().flatten()
            //     .filter_map(|r| match &r.content {
            //         RelationContent::Url(u) => Some(u.resource.to_string()),
            //         _ => None,
            //     })
            //     .collect(),
            name: none_if_empty(value.0.name),
            summary: None,
            ..Default::default()
        }
    }
}
