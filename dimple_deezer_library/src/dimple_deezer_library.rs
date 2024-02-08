use dimple_core::{collection::Collection, model::Model};
use serde::Deserialize;

/// https://developers.deezer.com/api
/// https://developers.deezer.com/guidelines
#[derive(Debug, Default)]
pub struct DeezerLibrary {
}

impl DeezerLibrary {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Collection for DeezerLibrary {
    fn name(&self) -> String {
        "Deezer".to_string()
    }

    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = Model>> {
        // let client = Client::builder()
        //     .https_only(true)
        //     .user_agent(dimple_core::USER_AGENT)
        //     .build().unwrap();
        // let url = format!("https://api.deezer.com/search/artist?q={}", query);
        // let response = client.get(url).send().unwrap();
        // let artist_resp = response.json::<DeezerResponse>().unwrap();
        // log::info!("Deezer found {} artists", artist_resp.total);
        // // let thumb = artist_resp.artistthumb.first()?;
        // // log::debug!("Downloading {}", &thumb.url);
        // // let thumb_resp = client.get(&thumb.url).send().ok()?;
        // // let bytes = thumb_resp.bytes().ok()?;
        // // image::load_from_memory(&bytes).ok()
        Box::new(vec![].into_iter())
    }

    // fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::DimpleArtist>> {
    //     Box::new(vec![].into_iter())
    // }

    // fn image(&self, _entity: &LibraryEntity) -> Option<image::DynamicImage> {
    //     None
    // }

    // fn sources(&self, _entity: &LibraryEntity) -> Box<dyn Iterator<Item = dimple_core::model::DimpleRecordingSource>> {
    //     if let LibraryEntity::Release(r) = _entity {
    //         let urls: Vec<_> = r.relations.iter()
    //             .filter_map(|r| {
    //                 if let DimpleRelationContent::Url(url) = &r.content {
    //                     return Some(url)
    //                 }
    //                 None
    //             })
    //             .collect();

    //     }
    //     Box::new(vec![].into_iter())
    // }
}


#[derive(Deserialize, Debug)]
struct DeezerResponse {
    #[serde(default)]
    data:  Vec<DeezerArtist>,

    #[serde(default)]
    total: u32,
}

/// https://developers.deezer.com/api/artist
#[derive(Deserialize, Debug)]
struct DeezerArtist {
    #[serde(default)]
    id: u32,

    #[serde(default)]
    name: String,

    #[serde(default)]
    picture: String,

    #[serde(default)]
    picture_big: String,
}

