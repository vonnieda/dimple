use std::env;

use dimple_core::{collection::{Collection, LibrarySupport}, model::Model};
use reqwest::blocking::Client;
use serde::Deserialize;

// TODO consider using https://crates.io/crates/fuzzy-matcher to try to find
// albums that might match the name of the artist to use as a back up for
// artist artwork.

// https://wiki.fanart.tv/General/personal%20api/
#[derive(Debug)]
pub struct FanartTvLibrary {
    api_key: String,
}

impl Default for FanartTvLibrary {
    fn default() -> Self {
        Self::new(&env::var("FANART_TV_API_KEY").unwrap_or_default())
    }
}

impl FanartTvLibrary {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}


#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ArtistResponse {
    name: String,
    artistthumb: Vec<ImageResponse>,
    musiclogo: Vec<ImageResponse>,
    hdmusiclogo: Vec<ImageResponse>,
    artistbackground: Vec<ImageResponse>,
    status: String,
    error_message: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ImageResponse {
    id: String,
    url: String,
    likes: String,
}

impl Collection for FanartTvLibrary {
    fn name(&self) -> String {
        "fanart.tv".to_string()
    }

    fn image(&self, entity: &Model) -> Option<image::DynamicImage> {
        match entity {
            Model::Artist(a) => {
                let client = Client::builder()
                    .https_only(true)
                    .user_agent(dimple_core::USER_AGENT)
                    .build().ok()?;
                let mbid = a.entity().mbid()?;
                let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", 
                    mbid, self.api_key);
                let request_token = LibrarySupport::start_request(self, &url);
                let response = client.get(url).send().ok()?;
                LibrarySupport::end_request(request_token, 
                    Some(response.status().as_u16()), 
                    response.content_length());
                let artist_resp = response.json::<ArtistResponse>().ok()?;
                // TODO see if we can get smaller images.
                // docs say you can add /preview, but when I added it to artistthumb
                // it didn't work (404)
                // https://fanart.tv/api-docs/api-v3/
                let thumb = artist_resp.artistthumb.first()
                    .or_else(|| artist_resp.artistbackground.first())
                    .or_else(|| artist_resp.hdmusiclogo.first())
                    .or_else(|| artist_resp.musiclogo.first())?;
                let request_token = LibrarySupport::start_request(self, &thumb.url);
                let thumb_resp = client.get(&thumb.url).send().ok()?;
                LibrarySupport::end_request(request_token, 
                    Some(thumb_resp.status().as_u16()),
                    thumb_resp.content_length());
                let bytes = thumb_resp.bytes().ok()?;
                image::load_from_memory(&bytes).ok()
            }
            _ => None,
        }
    }
}