use dimple_core::library::{Library, SearchResult};
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::prelude::*;

#[derive(Debug, Default)]
pub struct MusicBrainzLibrary {
}

impl MusicBrainzLibrary {
    pub fn new() -> Self {
        musicbrainz_rs::config::set_user_agent("DimpleMusicPlayer ( jason@vonnieda.org )");
        Self {
        }
    }
}

impl Library for MusicBrainzLibrary {
    fn name(&self) -> String {
        todo!()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = SearchResult>> {
        let query = query.to_string();
        // And releases, tracks, etc.
        let search_query = ArtistSearchQuery::query_builder()
                .artist(&query)
                .build();
        let results: Vec<SearchResult> = Artist::search(search_query)
            .execute().unwrap() // TODO error handling
            .entities
            .iter()
            .map(|src| {
                dimple_core::model::Artist {
                    name: src.name.clone(),
                    musicbrainz_id: src.id.clone(),
                    ..Default::default()
                }
            })
            .map(SearchResult::Artist)
            .collect();
        Box::new(results.into_iter())
    }
}

    // fn releases(&self) -> Receiver<Release> {
    //     let query = ReleaseGroupSearchQuery::query_builder()
    //     // .artist(query_str)
    //     .release_group(query_str)
    //     // .and()
    //     // .country("US")
    //     .build(); 
    // let mut results = musicbrainz_rs::entity::release_group::ReleaseGroup::search(query)
    //     .execute()
    //     .unwrap()
    //     .entities;
    // results.sort_by_key(|a| a.title.clone());
    // results.iter().for_each(|r| {
    //     let cover = r.get_coverart()
    //         .res_250()
    //         // .front()
    //         .execute();
    //     match cover {
    //         Ok(CoverartResponse::Json(json)) => {
    //             dbg!(json);
    //         },
    //         Ok(CoverartResponse::Url(url)) => {
    //             dbg!(url);
    //         },
    //         Err(_) => todo!(),
    //     }
    // });

    //     todo!()
    // }

