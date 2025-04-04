use std::{env};

use anyhow::{Error, Result};
use dimple_core::model::{Entity, Model, Dimage};
use dimple_librarian::plugin::{NetworkMode, Plugin, PluginContext, PluginSupport};
use serde::Deserialize;
// TODO consider using https://crates.io/crates/fuzzy-matcher to try to find
// albums that might match the name of the artist to use as a back up for
// artist artwork.

// TODO fanart.tv does have album art, but it seems like you have to query it
// by artist mbid, and I don't have a good way to do this with the plugin
// API right now.

// https://wiki.fanart.tv/General/personal%20api/
// https://fanart.tv/api-docs/api-v3/
// https://fanarttv.docs.apiary.io/#
// GET http://webservice.fanart.tv/v3/music/albums/id?api_key=6fa42b0ef3b5f3aab6a7edaa78675ac2
#[derive(Debug)]
pub struct FanartTvPlugin {
    api_key: String,
}

impl Default for FanartTvPlugin {
    fn default() -> Self {
        Self::new(&env::var("FANART_TV_API_KEY").expect("Missing FANART_TV_API_KEY environment variable."))
    }
}

impl FanartTvPlugin {
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

impl Plugin for FanartTvPlugin {
    fn name(&self) -> String {
        "fanart.tv".to_string()
    }
    
    fn list(
        &self,
        list_of: &dimple_core::model::Model,
        related_to: &Option<dimple_core::model::Model>,
        network_mode: dimple_librarian::plugin::NetworkMode,
        ctx: &PluginContext,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }

        match (list_of, related_to) {
            (Model::Dimage(_), Some(Model::Artist(artist))) => {
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;

                let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", 
                    mbid, self.api_key);
                let response = ctx.get(self, &url)?;
                let artist_resp = response.json::<ArtistResponse>()?;
                let thumb = artist_resp.artistthumb.first().ok_or_else(|| Error::msg("No images"))?;
                
                let thumb_resp = ctx.get(self, &thumb.url)?;
                let bytes = thumb_resp.bytes()?;
                let image = image::load_from_memory(&bytes)?;

                let mut dimage = Dimage::default();
                dimage.set_image(&image);
                
                Ok(Box::new(std::iter::once(dimage.model())))
            },
            _ => Ok(Box::new(std::iter::empty())),
        }
    }
}
