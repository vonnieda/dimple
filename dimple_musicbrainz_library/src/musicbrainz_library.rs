use dimple_core::library::{Library, LibraryEntity};
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::prelude::*;

#[derive(Debug, Default)]
pub struct MusicBrainzLibrary {
}

impl MusicBrainzLibrary {
    pub fn new() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_core::USER_AGENT);
        Self {
        }
    }
}

impl Library for MusicBrainzLibrary {
    fn name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
        let query = query.to_string();
        // TODO And releases, tracks, etc.
        let search_query = ArtistSearchQuery::query_builder()
                .artist(&query)
                .build();
        let results: Vec<LibraryEntity> = Artist::search(search_query)
            .execute().unwrap() // TODO error handling
            .entities
            .iter()
            .map(|src| {
                dimple_core::model::Artist {
                    mb: src.clone()
                }
            })
            .map(LibraryEntity::Artist)
            .collect();
        Box::new(results.into_iter())
    }

    fn fetch(&self, _entity: &LibraryEntity) -> Option<LibraryEntity> {
        match _entity {
            LibraryEntity::Artist(a) => {
                Artist::fetch()
                    .id(&a.mbid())
                    .with_aliases()
                    .with_annotations()
                    .with_genres()
                    .with_rating()
                    .with_tags()
                    .with_releases()
                    .with_url_relations()
                    .execute()
                    .ok()
                    .map(|src| {
                        dimple_core::model::Artist {
                            mb: src.clone()
                        }
                    })
                    .map(LibraryEntity::Artist)
                },
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(_) => todo!(),
            LibraryEntity::Track(_) => todo!(),
        }        
    }
}