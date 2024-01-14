use dimple_core::library::{Library, LibraryEntity, LibrarySupport};
use dimple_core::model::{DimpleGenre, DimpleArtist, DimpleReleaseGroup, DimpleRelation, DimpleRelationContent, DimpleUrl};
use image::DynamicImage;
use musicbrainz_rs::entity::CoverartResponse;
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::entity::relations::RelationContent;
use musicbrainz_rs::entity::release_group::ReleaseGroup;
use musicbrainz_rs::{prelude::*, FetchQuery};

#[derive(Debug, Default)]
pub struct MusicBrainzLibrary {
}

impl MusicBrainzLibrary {
    pub fn new() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_core::USER_AGENT);
        Self {
        }
    }

    pub fn get_coverart(&self, resp: CoverartResponse) -> Option<DynamicImage> {
        match resp {
            musicbrainz_rs::entity::CoverartResponse::Json(_) => todo!(),
            musicbrainz_rs::entity::CoverartResponse::Url(url) => {
                LibrarySupport::log_request(self, &url);
                reqwest::blocking::get(url).ok()
                    .map(|resp| resp.bytes().ok())?
                    .and_then(|bytes| image::load_from_memory(&bytes).ok())
            },    
        }
    }
}

// TODO all of the log_requests below are semi made up cause I can't get the
// real URL from the FetchQuery etc. I should at least change them to be correct
// according to the API spec.
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
                    &format!("http://musicbrainz.org/fetch/artist/{}", a.id));
                Artist::fetch()
                    .id(&a.id)
                    .with_aliases()
                    .with_annotations()
                    .with_genres()
                    .with_rating()
                    .with_tags()
                    .with_releases()
                    .with_release_groups()
                    .with_url_relations()
                    .execute()
                    .ok()
                    .map(|src| DimpleArtist::from(ArtistConverter::from(src.clone())))
                    .map(LibraryEntity::Artist)        
                },
            LibraryEntity::Genre(_) => None,
            LibraryEntity::Release(_) => None,
            LibraryEntity::Track(_) => None,
        }        
    }

    fn image(&self, _entity: &LibraryEntity) -> Option<image::DynamicImage> {
        match _entity {
            LibraryEntity::Release(r) => {
                LibrarySupport::log_request(self, 
                    &format!("http://coverartarchive.org/{}", r.id));                
                let mb = ReleaseGroup {
                    id: r.id.to_string(),
                    ..Default::default()
                };
                mb.get_coverart()
                    .front()
                    .execute()
                    .ok()
                    .map(|resp| self.get_coverart(resp))?
            },
            LibraryEntity::Artist(_) => None,
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
            bio: None,
            // TODO this is always going to be Some even if there are None
            genres: Some(value.0.genres.iter()
                .flatten()
                .map(|f| f.to_owned())
                .map(|f| DimpleGenre::from(GenreConverter::from(f)))
                .collect()),
            release_groups: Some(value.0.release_groups.iter()
                .flatten()
                .map(|f| f.to_owned())
                .map(|f| DimpleReleaseGroup::from(ReleaseGroupConverter::from(f)))
                .collect()),
            relations: Some(value.0.relations.iter()
                .flatten()
                .map(|f| f.to_owned())
                .map(|f| DimpleRelation::from(RelationConverter::from(f)))
                .collect()),
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
            description: None,
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
            RelationContent::Artist(_) => todo!(),
            RelationContent::Area(_) => todo!(),
            RelationContent::Event(_) => todo!(),
            RelationContent::Label(_) => todo!(),
            RelationContent::Place(_) => todo!(),
            RelationContent::Recording(_) => todo!(),
            RelationContent::Release(_) => todo!(),
            RelationContent::ReleaseGroup(_) => todo!(),
            RelationContent::Series(_) => todo!(),
            RelationContent::Work(_) => todo!(),
        }
    }
}
