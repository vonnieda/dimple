use std::sync::mpsc::Receiver;

use dimple_core::{model::{Release, Track, Image}, library::Library};
use image::DynamicImage;

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
    
    fn releases(&self) -> Receiver<Release> {
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

        todo!()
    }

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        todo!()
    }

    fn stream(&self, _track: &Track) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn merge_release(&self, library: &dyn Library, release: &Release) -> Result<(), String> {
        todo!()
    }
}
