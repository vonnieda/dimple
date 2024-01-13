use dimple_core::{library::{Library, LibraryEntity}, model::DimpleRelationContent};
use reqwest::blocking::Client;
use serde::Deserialize;

/// https://developers.deezer.com/api
#[derive(Debug, Default)]
pub struct DeezerLibrary {
}

impl DeezerLibrary {
    pub fn new() -> Self {
        Self {
        }
    }
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

impl Library for DeezerLibrary {
    fn name(&self) -> String {
        "Deezer".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
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

    fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::DimpleArtist>> {
        Box::new(vec![].into_iter())
    }

    fn image(&self, entity: &LibraryEntity) -> Option<image::DynamicImage> {
        match entity {
            LibraryEntity::Artist(a) => {
                let a = a.clone();
                a.relations?.clone().iter()
                    .for_each(|rel| {
                        if let DimpleRelationContent::Url(con) = &rel.content {
                            dbg!(&con.resource);
                        }
                    });
                None
            },
            LibraryEntity::Genre(_) => None,
            LibraryEntity::Release(_) => None,
            LibraryEntity::Track(_) => None,           
        }
    }
}