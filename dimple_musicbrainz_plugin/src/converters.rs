use dimple_core::model::Artist;
use dimple_core::model::ArtistCredit;
use dimple_core::model::Medium;
use dimple_core::model::Recording;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use musicbrainz_rs::entity::relations::RelationContent;
use dimple_core::model::{Genre, KnownIds};


// Note that in the converters below ..Default should never be used. If a Default
// is temporarily needed it can be specified on the field itself, but not
// the entire struct. This is to help avoid skipping fields when new ones
// are added.


fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    }
    else {
        Some(s)
    }
}

pub struct ArtistConverter(musicbrainz_rs::entity::artist::Artist);

impl From<musicbrainz_rs::entity::artist::Artist> for ArtistConverter {
    fn from(value: musicbrainz_rs::entity::artist::Artist) -> Self {
        ArtistConverter(value)
    }
}

impl From<ArtistConverter> for dimple_core::model::Artist {
    fn from(value: ArtistConverter) -> Self {
        dimple_core::model::Artist {
            country: value.0.country,
            disambiguation: none_if_empty(value.0.disambiguation),
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id.clone()),
                ..Default::default()
            },
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(std::iter::once(value.0.id.clone())
                    .map(|mbid| format!("https://musicbrainz.org/artist/{}", mbid)))
                .collect(),
            name: none_if_empty(value.0.name),
            summary: None,
        }
    }
}

pub struct ArtistCreditConverter(musicbrainz_rs::entity::artist_credit::ArtistCredit);

impl From<musicbrainz_rs::entity::artist_credit::ArtistCredit> for ArtistCreditConverter {
    fn from(value: musicbrainz_rs::entity::artist_credit::ArtistCredit) -> Self {
        ArtistCreditConverter(value)
    }
}

impl From<ArtistCreditConverter> for dimple_core::model::ArtistCredit {
    fn from(value: ArtistCreditConverter) -> Self {
        ArtistCredit {
            artist: Artist::from(ArtistConverter::from(value.0.artist)),
            join_phrase: value.0.joinphrase,
            key: None,
            name: Some(value.0.name),
        }
    }
}

impl From<ArtistCreditConverter> for dimple_core::model::Artist {
    fn from(value: ArtistCreditConverter) -> Self {
        Artist::from(ArtistConverter::from(value.0.artist))
    }
}

pub struct ReleaseGroupConverter(musicbrainz_rs::entity::release_group::ReleaseGroup);

impl From<musicbrainz_rs::entity::release_group::ReleaseGroup> for ReleaseGroupConverter {
    fn from(value: musicbrainz_rs::entity::release_group::ReleaseGroup) -> Self {
        ReleaseGroupConverter(value)
    }
}

impl From<ReleaseGroupConverter> for dimple_core::model::ReleaseGroup {
    fn from(value: ReleaseGroupConverter) -> Self {
        dimple_core::model::ReleaseGroup {
            annotation: value.0.annotation,
            artist_credits: value.0.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            disambiguation: none_if_empty(value.0.disambiguation),
            first_release_date: value.0.first_release_date.map(|f| f.to_string()),
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id.clone()),
                ..Default::default()
            },
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(std::iter::once(value.0.id.clone())
                    .map(|mbid| format!("https://musicbrainz.org/release-group/{}", mbid)))
                .collect(),
            secondary_types: value.0.secondary_types.iter()
                .map(|f| format!("{:?}", f))
                .collect(),
            primary_type: value.0.primary_type.map(|f| format!("{:?}", f)),
            summary: None,
            title: none_if_empty(value.0.title),
        }
    }
}

pub struct ReleaseConverter(musicbrainz_rs::entity::release::Release);

impl From<musicbrainz_rs::entity::release::Release> for ReleaseConverter {
    fn from(value: musicbrainz_rs::entity::release::Release) -> Self {
        ReleaseConverter(value)
    }
}

impl From<ReleaseConverter> for dimple_core::model::Release {
    fn from(value: ReleaseConverter) -> Self {
        dimple_core::model::Release {
            artist_credits: value.0.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            barcode: value.0.barcode,
            country: value.0.country,
            date: value.0.date.map(|f| f.to_string()),
            disambiguation: value.0.disambiguation,
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id.clone()),
                ..Default::default()
            },
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .chain(std::iter::once(value.0.id.clone())
                    .map(|mbid| format!("https://musicbrainz.org/release/{}", mbid)))
                .collect(),
            title: none_if_empty(value.0.title),
            packaging: value.0.packaging.map(|f| format!("{:?}", f)),
            primary_type: value.0.release_group.clone().and_then(|rg| rg.primary_type).and_then(|pt| Some(format!("{:?}", pt))),
            release_group: value.0.release_group
                .map(|f| ReleaseGroup::from(ReleaseGroupConverter::from(f.to_owned()))).unwrap(),
            status: value.0.status.map(|f| format!("{:?}", f)),
            quality: value.0.quality.map(|f| format!("{:?}", f)),
            summary: None,
            media: value.0.media.iter().flatten()
                .map(|f| Medium::from(MediumConverter::from(f.to_owned())))
                .collect(),
        }
    }
}

pub struct MediumConverter(musicbrainz_rs::entity::release::Media);

impl From<musicbrainz_rs::entity::release::Media> for MediumConverter {
    fn from(value: musicbrainz_rs::entity::release::Media) -> Self {
        MediumConverter(value)
    }
}

impl From<MediumConverter> for dimple_core::model::Medium {
    fn from(value: MediumConverter) -> Self {
        dimple_core::model::Medium {
            key: None,
            title: value.0.title,            
            disc_count: value.0.disc_count,
            format: value.0.format,
            position: value.0.position,
            track_count: Some(value.0.track_count),
            tracks: value.0.tracks.iter().flatten()
                .map(|f| Track::from(TrackConverter::from(f.to_owned())))
                .collect(),
        }
    }
}

pub struct TrackConverter(musicbrainz_rs::entity::release::Track);

impl From<musicbrainz_rs::entity::release::Track> for TrackConverter {
    fn from(value: musicbrainz_rs::entity::release::Track) -> Self {
        TrackConverter(value)
    }
}

impl From<TrackConverter> for dimple_core::model::Track {
    fn from(value: TrackConverter) -> Self {
        dimple_core::model::Track {
            artist_credits: value.0.recording.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            genres: value.0.recording.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            length: value.0.length,
            number: u32::from_str_radix(&value.0.number, 10).ok(),
            position: Some(value.0.position),
            recording: Recording::from(RecordingConverter::from(value.0.recording.to_owned())),
            title: none_if_empty(value.0.title),
        }
    }
}

pub struct RecordingConverter(musicbrainz_rs::entity::recording::Recording);

impl From<musicbrainz_rs::entity::recording::Recording> for RecordingConverter {
    fn from(value: musicbrainz_rs::entity::recording::Recording) -> Self {
        RecordingConverter(value)
    }
}

impl From<RecordingConverter> for dimple_core::model::Recording {
    fn from(value: RecordingConverter) -> Self {
        dimple_core::model::Recording {
            annotation: value.0.annotation,
            artist_credits: value.0.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            disambiguation: value.0.disambiguation,
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            isrc: value.0.isrcs.into_iter().flatten().next(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            length: value.0.length,
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .collect(),
            summary: None,
            title: none_if_empty(value.0.title),
        }
    }
}

pub struct GenreConverter(musicbrainz_rs::entity::genre::Genre);

impl From<musicbrainz_rs::entity::genre::Genre> for GenreConverter {
    fn from(value: musicbrainz_rs::entity::genre::Genre) -> Self {
        GenreConverter(value)
    }
}

impl From<GenreConverter> for dimple_core::model::Genre {
    fn from(value: GenreConverter) -> Self {
        dimple_core::model::Genre {
            disambiguation: None,
            key: None,
            // known_ids: KnownIds {
            //     musicbrainz_id: Some(value.0.id),
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