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
        todo!()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
        let query = query.to_string();
        // And releases, tracks, etc.
        let search_query = ArtistSearchQuery::query_builder()
                .artist(&query)
                .build();
        let results: Vec<LibraryEntity> = Artist::search(search_query)
            .execute().unwrap() // TODO error handling
            .entities
            .iter()
            .map(|src| {
                dimple_core::model::Artist {
                    name: src.name.clone(),
                    mbid: Some(src.id.clone()),
                    ..Default::default()
                }
            })
            .map(LibraryEntity::Artist)
            .collect();
        Box::new(results.into_iter())
    }

    fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::Artist>> {
        Box::new(vec![].into_iter())
    }

    fn image(&self, image: &dimple_core::model::Image) -> Option<image::DynamicImage> {
        None
    }
}