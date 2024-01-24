use std::env;

use dimple_core::library::{Library, DimpleEntity, LibrarySupport};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug)]
pub struct TheAudioDbLibrary {
    api_key: String,
}

impl Default for TheAudioDbLibrary {
    fn default() -> Self {
        Self::new(&env::var("TADB_API_KEY").unwrap_or_default())
    }
}


impl TheAudioDbLibrary {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ArtistsResponse {
    artists: Vec<ArtistResponse>,
}


#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ArtistResponse {
    idArtist: String,
    strArtist: String,
    strBiographyEN: String,
    strArtistThumb: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ImageResponse {
    id: String,
    url: String,
    likes: String,
}

// https://www.theaudiodb.com/free_music_api
impl Library for TheAudioDbLibrary {
    fn name(&self) -> String {
        "TheAudioDB".to_string()
    }

    // TODO add bio

    fn image(&self, entity: &DimpleEntity) -> Option<image::DynamicImage> {
        match entity {
            // https://www.theaudiodb.com/api/v1/json/api_key/artist-mb.php?i=1d86a19b-8ddd-448c-a815-4f41350bea53
            DimpleEntity::Artist(a) => {
                let client = Client::builder()
                    .https_only(true)
                    .user_agent(dimple_core::USER_AGENT)
                    .build().ok()?;

                let mbid = a.id.to_string();

                let url = format!("https://www.theaudiodb.com/api/v1/json/{}/artist-mb.php?i={}", 
                    self.api_key, mbid);
                let request_token = LibrarySupport::start_request(self, &url);
                let response = client.get(url).send().ok()?;
                LibrarySupport::end_request(request_token, 
                    Some(response.status().as_u16()), 
                    response.content_length());
                let artists_resp = response.json::<ArtistsResponse>().ok()?;

                let artist_thumbnail_url = artists_resp.artists.first()?
                    .strArtistThumb.clone();
                if artist_thumbnail_url.is_empty() {
                    return None;
                }

                let request_token = LibrarySupport::start_request(self, &artist_thumbnail_url);
                let thumb_resp = client.get(&artist_thumbnail_url).send().ok()?;
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