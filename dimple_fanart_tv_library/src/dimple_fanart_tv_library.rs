use dimple_core::{library::{Library, LibraryEntity}, model::Artist};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct FanartTvLibrary {
}

impl FanartTvLibrary {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Deserialize, Debug)]
struct ArtistResponse {
    #[serde(default)]
    name: String,
    #[serde(default)]
    artistthumb: Vec<ImageResponse>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    error_message: String,
}

#[derive(Deserialize, Debug)]
struct ImageResponse {
    id: String,
    url: String,
    likes: String,
}

impl Library for FanartTvLibrary {
    fn name(&self) -> String {
        "fanart.tv".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
        Box::new(vec![].into_iter())
    }

    fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::Artist>> {
        Box::new(vec![].into_iter())
    }

    fn image(&self, entity: &LibraryEntity) -> Option<image::DynamicImage> {
        match entity {
            LibraryEntity::Artist(a) => {
                let client = Client::builder()
                    .https_only(true)
                    .user_agent(dimple_core::USER_AGENT)
                    .build().ok()?;
                let api_key = "55b9ef19f6822b9f835c97426d435d72";
                let mbid = a.mbid.clone()?;
                let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", mbid, api_key);
                let response = client.get(url).send().ok()?;
                let artist_resp = response.json::<ArtistResponse>().ok()?;
                let thumb = artist_resp.artistthumb.first()?;
                log::debug!("Downloading {}", &thumb.url);
                let thumb_resp = client.get(&thumb.url).send().ok()?;
                let bytes = thumb_resp.bytes().ok()?;
                image::load_from_memory(&bytes).ok()
            }
            LibraryEntity::Genre(_) => None,
            LibraryEntity::Release(_) => None,
            LibraryEntity::Track(_) => None,
        }
    }
}