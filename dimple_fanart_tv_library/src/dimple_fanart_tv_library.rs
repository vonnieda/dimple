use dimple_core::library::{Library, LibraryEntity, LibrarySupport};
use reqwest::blocking::Client;
use serde::Deserialize;

// https://wiki.fanart.tv/General/personal%20api/
#[derive(Debug, Default)]
pub struct FanartTvLibrary {
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

impl Library for FanartTvLibrary {
    fn name(&self) -> String {
        "fanart.tv".to_string()
    }

    fn image(&self, entity: &LibraryEntity) -> Option<image::DynamicImage> {
        match entity {
            LibraryEntity::Artist(a) => {
                let client = Client::builder()
                    .https_only(true)
                    .user_agent(dimple_core::USER_AGENT)
                    .build().ok()?;
                let api_key = "55b9ef19f6822b9f835c97426d435d72";
                let mbid = a.id.to_string();
                let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", mbid, api_key);
                LibrarySupport::log_request(self, &url);
                let response = client.get(url).send().ok()?;
                let artist_resp = response.json::<ArtistResponse>().ok()?;
                let thumb = artist_resp.artistthumb.first()
                    .or_else(|| artist_resp.artistbackground.first())
                    .or_else(|| artist_resp.hdmusiclogo.first())
                    .or_else(|| artist_resp.musiclogo.first())?;
                LibrarySupport::log_request(self, &thumb.url);
                let thumb_resp = client.get(&thumb.url).send().ok()?;
                let bytes = thumb_resp.bytes().ok()?;
                image::load_from_memory(&bytes).ok()
            }
            // Seems like it only supports artists by mbid
            LibraryEntity::Genre(_) => None,
            LibraryEntity::ReleaseGroup(_) => None,
            LibraryEntity::Release(_) => None,
            LibraryEntity::Track(_) => None,
        }
    }
}