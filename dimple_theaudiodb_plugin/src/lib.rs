use std::env;

use anyhow::{Error, Result};
use dimple_core::model::{Entity, Model, Picture};
use dimple_librarian::plugin::{NetworkMode, Plugin, PluginSupport};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug)]
pub struct TheAudioDbPlugin {
    api_key: String,
}

impl Default for TheAudioDbPlugin {
    fn default() -> Self {
        Self::new(&env::var("TADB_API_KEY").unwrap_or_default())
    }
}


impl TheAudioDbPlugin {
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
// TODO add bio
// TODO add release groups (albums)
impl Plugin for TheAudioDbPlugin {
    fn name(&self) -> String {
        "TheAudioDB".to_string()
    }

    fn list(
        &self,
        list_of: &dimple_core::model::Model,
        related_to: &Option<dimple_core::model::Model>,
        network_mode: dimple_librarian::plugin::NetworkMode,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }

        match (list_of, related_to) {
            (Model::Picture(_), Some(Model::Artist(artist))) => {
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;

                let client = Client::builder()
                    .https_only(true)
                    .user_agent(dimple_librarian::plugin::USER_AGENT)
                    .build()?;

                let url = format!("https://www.theaudiodb.com/api/v1/json/{}/artist-mb.php?i={}", 
                    self.api_key, mbid);
                let request_token = PluginSupport::start_request(self, &url);
                let response = client.get(url).send()?;
                PluginSupport::end_request(request_token, 
                    Some(response.status().as_u16()), 
                    response.content_length());
                let artists_resp = response.json::<ArtistsResponse>()?;

                let artist_thumbnail_url = artists_resp.artists.first().ok_or(Error::msg("no thumbnail"))?
                    .strArtistThumb.clone();
                if artist_thumbnail_url.is_empty() {
                    return Ok(Box::new(std::iter::empty()))
                }

                let request_token = PluginSupport::start_request(self, &artist_thumbnail_url);
                let thumb_resp = client.get(&artist_thumbnail_url).send()?;
                PluginSupport::end_request(request_token, 
                    Some(thumb_resp.status().as_u16()),
                    thumb_resp.content_length());
                let bytes = thumb_resp.bytes()?;
                let image = image::load_from_memory(&bytes)?;

                let mut picture = Picture::default();
                picture.set_image(&image);
                Ok(Box::new(std::iter::once(picture.model())))
            },
            _ => Ok(Box::new(std::iter::empty())),
        }
    }
}