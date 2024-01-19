use dimple_core::library::{Library, LibraryEntity, LibrarySupport};
use dimple_core::model::{DimpleGenre, DimpleArtist, DimpleReleaseGroup, DimpleRelation, DimpleRelationContent, DimpleUrl, DimpleRelease, DimpleMedium, DimpleTrack, DimpleRecording};
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::entity::recording::Recording;
use musicbrainz_rs::entity::relations::RelationContent;
use musicbrainz_rs::entity::release::{Release, Track};
use musicbrainz_rs::entity::release_group::ReleaseGroup;
use musicbrainz_rs::{prelude::*, FetchQuery};

#[derive(Debug, Default)]
pub struct MusicBrainzLibrary {
}

// TODO implement high level rate limiting and paralleling
impl MusicBrainzLibrary {
    pub fn new() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_core::USER_AGENT);
        Self {
        }
    }
}

// TODO all of the log_requests below are semi made up cause I can't get the
// real URL from the FetchQuery etc.
impl Library for MusicBrainzLibrary {
    fn name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
        let query = query.to_string();

        LibrarySupport::log_request(self, 
            &format!("http://musicbrainz.org/search/artist/{}", &query));

        // TODO And releases, tracks, etc.
        let search_query = ArtistSearchQuery::query_builder()
            .artist(&query)
            .build();
        let results: Vec<LibraryEntity> = Artist::search(search_query)
            .execute().unwrap() // TODO error handling
            .entities
            .iter()
            .map(|src| dimple_core::model::DimpleArtist::from(ArtistConverter::from(src.clone())))
            .map(LibraryEntity::Artist)
            .collect();
        Box::new(results.into_iter())
    }

    fn fetch(&self, _entity: &LibraryEntity) -> Option<LibraryEntity> {
        match _entity {
            LibraryEntity::Artist(a) => {
                LibrarySupport::log_request(self, 
                    &format!("https://musicbrainz.org/ws/2/artist/{}?inc=aliases%20release-groups%20releases%20release-group-rels%20release-rels&fmt=json", a.id));
                Artist::fetch().id(&a.id)
                    .with_aliases().with_annotations().with_genres().with_rating()
                    .with_tags().with_release_groups().with_url_relations()
                    .execute()
                    .inspect_err(|f| log::error!("{}", f))
                    .ok()
                    .inspect(|src| log::debug!("{:?}", src))
                    .map(|src| DimpleArtist::from(ArtistConverter::from(src.clone())))
                    .map(LibraryEntity::Artist)        
            },
            LibraryEntity::ReleaseGroup(r) => {
                LibrarySupport::log_request(self, 
                    &format!("https://musicbrainz.org/ws/2/release-group/{}?inc=aliases%20artists%20releases%20release-group-rels%20release-rels%20url-rels&fmt=json", r.id));
                ReleaseGroup::fetch().id(&r.id)
                    .with_aliases().with_annotations().with_artists()
                    .with_genres().with_ratings().with_releases().with_tags()
                    .with_url_relations()
                    .execute()
                    .inspect_err(|f| log::error!("{}", f))
                    .ok()
                    .inspect(|src| log::debug!("{:?}", src))
                    .map(|src| DimpleReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
                    .map(LibraryEntity::ReleaseGroup)        
            },
            LibraryEntity::Release(r) => {
                LibrarySupport::log_request(self, 
                    &format!("https://musicbrainz.org/ws/2/release/{}?inc=aliases%20artist-credits%20artist-rels%20artists%20genres%20labels%20ratings%20recording-rels%20recordings%20release-groups%20release-group-rels%20tags%20release-rels%20url-rels%20work-level-rels%20work-rels&fmt=json", r.id));
                Release::fetch().id(&r.id)
                    .with_aliases().with_annotations().with_artist_credits()
                    .with_artists().with_genres().with_labels().with_ratings()
                    .with_recordings().with_release_groups().with_tags()
                    .with_url_relations()
                    .execute()
                    .inspect_err(|f| log::error!("{}", f))
                    .ok()
                    .inspect(|src| log::debug!("{:?}", src))
                    .map(|src| DimpleRelease::from(ReleaseConverter::from(src.clone())))
                    .map(LibraryEntity::Release)        
            },
            LibraryEntity::Recording(r) => {
                LibrarySupport::log_request(self, 
                    &format!("https://musicbrainz.org/ws/2/recording/{}?inc=aliases%20artist-credits%20artist-rels%20artists%20genres%20labels%20ratings%20recording-rels%20recordings%20release-groups%20release-group-rels%20tags%20release-rels%20url-rels%20work-level-rels%20work-rels&fmt=json", r.id));
                Recording::fetch().id(&r.id)
                    .with_aliases()
                    .with_annotations()
                    // .with_artist_credits()
                    .with_artists()
                    .with_genres()
                    .with_isrcs()
                    // .with_labels()
                    .with_ratings()
                    // .with_recordings()
                    .with_releases()
                    .with_tags()
                    .with_url_relations()
                    .execute()
                    .inspect_err(|f| log::error!("{}", f))
                    .ok()
                    .inspect(|src| log::debug!("{:?}", src))
                    .map(|src| DimpleRecording::from(RecordingConverter::from(src.clone())))
                    .map(LibraryEntity::Recording)        
            },
            LibraryEntity::Genre(_) => None,
            LibraryEntity::Track(_) => None,
        }        
    }
}

pub struct ArtistConverter(musicbrainz_rs::entity::artist::Artist);

impl From<musicbrainz_rs::entity::artist::Artist> for ArtistConverter {
    fn from(value: musicbrainz_rs::entity::artist::Artist) -> Self {
        ArtistConverter(value)
    }
}

impl From<ArtistConverter> for dimple_core::model::DimpleArtist {
    fn from(value: ArtistConverter) -> Self {
        dimple_core::model::DimpleArtist {
            id: value.0.id,
            name: value.0.name,
            disambiguation: value.0.disambiguation,
            genres: value.0.genres.iter().flatten()
                .map(|f| DimpleGenre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            release_groups: value.0.release_groups.iter().flatten()
                .map(|f| DimpleReleaseGroup::from(ReleaseGroupConverter::from(f.to_owned())))
                .collect(),
            relations: value.0.relations.iter().flatten()
                .map(|f| DimpleRelation::from(RelationConverter::from(f.to_owned())))
                .collect(),
            summary: Default::default(),
            fetched: Default::default(),
        }
    }
}

pub struct ReleaseGroupConverter(musicbrainz_rs::entity::release_group::ReleaseGroup);

impl From<musicbrainz_rs::entity::release_group::ReleaseGroup> for ReleaseGroupConverter {
    fn from(value: musicbrainz_rs::entity::release_group::ReleaseGroup) -> Self {
        ReleaseGroupConverter(value)
    }
}

impl From<ReleaseGroupConverter> for dimple_core::model::DimpleReleaseGroup {
    fn from(value: ReleaseGroupConverter) -> Self {
        dimple_core::model::DimpleReleaseGroup {
            id: value.0.id,
            title: value.0.title,
            disambiguation: value.0.disambiguation,
            primary_type: value.0.primary_type.map(|f| format!("{:?}", f)).unwrap_or("".to_string()),
            first_release_date: value.0.first_release_date.map(|f| f.to_string()).unwrap_or("".to_string()),
            genres: value.0.genres.iter()
                .flatten()
                .map(|f| DimpleGenre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            releases: value.0.releases.iter()
                .flatten()
                .map(|f| DimpleRelease::from(ReleaseConverter::from(f.to_owned())))
                .collect(),
            relations: value.0.relations.iter()
                .flatten()
                .map(|f| DimpleRelation::from(RelationConverter::from(f.to_owned())))
                .collect(),
            artists: value.0.artist_credit.iter()
                .flatten()
                .map(|f| DimpleArtist::from(ArtistCreditConverter::from(f.to_owned())))
                .collect(),
            summary: Default::default(),
            fetched: Default::default(),
        }
    }
}

pub struct ReleaseConverter(musicbrainz_rs::entity::release::Release);

impl From<musicbrainz_rs::entity::release::Release> for ReleaseConverter {
    fn from(value: musicbrainz_rs::entity::release::Release) -> Self {
        ReleaseConverter(value)
    }
}

impl From<ReleaseConverter> for dimple_core::model::DimpleRelease {
    fn from(value: ReleaseConverter) -> Self {
        dimple_core::model::DimpleRelease {
            id: value.0.id,
            title: value.0.title,

            artists: value.0.artist_credit.iter().flatten()
                .map(|f| DimpleArtist::from(ArtistCreditConverter::from(f.to_owned())))
                .collect(),
            barcode: value.0.barcode.map(|f| f.to_string()).unwrap_or("".to_string()),
            country: value.0.country.unwrap_or("".to_string()),
            date: value.0.date.map(|f| f.to_string()).unwrap_or("".to_string()),
            disambiguation: value.0.disambiguation.unwrap_or("".to_string()),
            genres: value.0.genres.iter().flatten()
                .map(|f| DimpleGenre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            packaging: value.0.packaging.map(|f| format!("{:?}", f)).unwrap_or("".to_string()),
            status: value.0.status.map(|f| format!("{:?}", f)).unwrap_or("".to_string()),
            // // TODO unwrap
            // release_group: value.0.release_group
            //     .map(|f| DimpleReleaseGroup::from(ReleaseGroupConverter::from(f.to_owned()))).unwrap(),
            relations: value.0.relations.iter().flatten()
                .map(|f| DimpleRelation::from(RelationConverter::from(f.to_owned())))
                .collect(),
            media: value.0.media.iter().flatten()
                .map(|f| DimpleMedium::from(MediumConverter::from(f.to_owned())))
                .collect(),

            release_group: Default::default(),
            summary: Default::default(),
            fetched: Default::default(),
        }
    }
}

pub struct GenreConverter(musicbrainz_rs::entity::genre::Genre);

impl From<musicbrainz_rs::entity::genre::Genre> for GenreConverter {
    fn from(value: musicbrainz_rs::entity::genre::Genre) -> Self {
        GenreConverter(value)
    }
}

impl From<GenreConverter> for dimple_core::model::DimpleGenre {
    fn from(value: GenreConverter) -> Self {
        dimple_core::model::DimpleGenre {
            name: value.0.name,
            count: value.0.count,
            summary: Default::default(),
            fetched: Default::default(),
        }
    }
}

pub struct RelationConverter(musicbrainz_rs::entity::relations::Relation);

impl From<musicbrainz_rs::entity::relations::Relation> for RelationConverter {
    fn from(value: musicbrainz_rs::entity::relations::Relation) -> Self {
        RelationConverter(value)
    }
}

impl From<RelationConverter> for dimple_core::model::DimpleRelation {
    fn from(value: RelationConverter) -> Self {
        Self {
            content: RelationContentConverter::from(value.0.content.clone()).into(),
        }
    }
}

pub struct RelationContentConverter(musicbrainz_rs::entity::relations::RelationContent);

impl From<musicbrainz_rs::entity::relations::RelationContent> for RelationContentConverter {
    fn from(value: musicbrainz_rs::entity::relations::RelationContent) -> Self {
        RelationContentConverter(value)
    }
}

impl From<RelationContentConverter> for dimple_core::model::DimpleRelationContent {
    fn from(value: RelationContentConverter) -> Self {
        match value.0 {
            RelationContent::Url(u) => {
                DimpleRelationContent::Url(DimpleUrl {
                    id: u.id,
                    resource: u.resource,
                })
            },
            _ => todo!()
        }
    }
}

pub struct ArtistCreditConverter(musicbrainz_rs::entity::artist_credit::ArtistCredit);

impl From<musicbrainz_rs::entity::artist_credit::ArtistCredit> for ArtistCreditConverter {
    fn from(value: musicbrainz_rs::entity::artist_credit::ArtistCredit) -> Self {
        ArtistCreditConverter(value)
    }
}

impl From<ArtistCreditConverter> for dimple_core::model::DimpleArtist {
    fn from(value: ArtistCreditConverter) -> Self {
        DimpleArtist::from(ArtistConverter::from(value.0.artist))
    }
}

pub struct MediumConverter(musicbrainz_rs::entity::release::Media);

impl From<musicbrainz_rs::entity::release::Media> for MediumConverter {
    fn from(value: musicbrainz_rs::entity::release::Media) -> Self {
        MediumConverter(value)
    }
}

impl From<MediumConverter> for dimple_core::model::DimpleMedium {
    fn from(value: MediumConverter) -> Self {
        dimple_core::model::DimpleMedium {
            title: value.0.title.unwrap_or_default(),
            
            disc_count: value.0.disc_count.unwrap_or_default(),
            format: value.0.format.unwrap_or_default(),
            position: value.0.position.unwrap_or_default(),
            tracks: value.0.tracks.iter().flatten()
                .map(|f| DimpleTrack::from(TrackConverter::from(f.to_owned())))
                .collect(),
            track_count: value.0.track_count,

            fetched: Default::default(),
        }
    }
}

pub struct TrackConverter(musicbrainz_rs::entity::release::Track);

impl From<musicbrainz_rs::entity::release::Track> for TrackConverter {
    fn from(value: musicbrainz_rs::entity::release::Track) -> Self {
        TrackConverter(value)
    }
}

impl From<TrackConverter> for dimple_core::model::DimpleTrack {
    fn from(value: TrackConverter) -> Self {
        dimple_core::model::DimpleTrack {
            id: value.0.id,
            title: value.0.title,
            number: value.0.number,
            length: value.0.length.unwrap_or_default(),
            position: value.0.position,
            recording: DimpleRecording::from(RecordingConverter::from(value.0.recording)),

            fetched: Default::default()
        }
    }
}

pub struct RecordingConverter(musicbrainz_rs::entity::recording::Recording);

impl From<musicbrainz_rs::entity::recording::Recording> for RecordingConverter {
    fn from(value: musicbrainz_rs::entity::recording::Recording) -> Self {
        RecordingConverter(value)
    }
}

impl From<RecordingConverter> for dimple_core::model::DimpleRecording {
    fn from(value: RecordingConverter) -> Self {
        dimple_core::model::DimpleRecording {
            id: value.0.id,
            title: value.0.title,

            annotation: value.0.annotation.unwrap_or_default(),
            disambiguation: value.0.disambiguation.unwrap_or_default(),
            genres: value.0.genres.iter().flatten()
                .map(|f| DimpleGenre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            isrcs: value.0.isrcs.unwrap_or_default(),
            length: value.0.length.unwrap_or_default(),
            relations: value.0.relations.iter().flatten()
                .map(|f| DimpleRelation::from(RelationConverter::from(f.to_owned())))
                .collect(),
            releases: value.0.releases.iter()
                .flatten()
                .map(|f| DimpleRelease::from(ReleaseConverter::from(f.to_owned())))
                .collect(),
            artist_credits: value.0.artist_credit.iter()
                .flatten()
                .map(|f| DimpleArtist::from(ArtistCreditConverter::from(f.to_owned())))
                .collect(),

            summary: Default::default(),
            fetched: Default::default()
        }
    }
}

