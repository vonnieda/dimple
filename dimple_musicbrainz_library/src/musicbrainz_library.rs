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

    // https://musicbrainz.org/doc/MusicBrainz_API/Search
    // <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    // <metadata created="2017-03-12T16:54:57.165Z" xmlns="http://musicbrainz.org/ns/mmd-2.0#" xmlns:ext="http://musicbrainz.org/ns/ext#-2.0">
    //   <artist-list count="11" offset="0">
    //     <artist id="e56fd97e-c18f-4e5e-9b4d-f9fc21b4973f" type="Group" ext:score="100">
    //       <name>Fred</name>
    //       <sort-name>Fred</sort-name>
    //       <country>US</country>
    //       <area id="489ce91b-6658-3307-9877-795b68554c98">
    //         <name>United States</name>
    //         <sort-name>United States</sort-name>
    //       </area>
    //       <begin-area id="489ce91b-6658-3307-9877-795b68554c98">
    //         <name>United States</name>
    //         <sort-name>United States</sort-name>
    //       </begin-area>
    //       <disambiguation>US progressive rock band</disambiguation>
    //       <life-span>
    //         <begin>1969</begin>
    //         <end>1974</end>
    //         <ended>true</ended>
    //       </life-span>
    //     </artist>    
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

    fn image(&self, entity: &LibraryEntity) -> Option<image::DynamicImage> {
        None
    }
}