use std::env;

use anyhow::{anyhow, Error, Result};
use image::DynamicImage;
use serde::Deserialize;

use crate::{library::Library, model::{dimage::DimageKind, Artist, Dimage, Model}};

use super::{plugin::Plugin, plugins::Plugins};
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

impl Plugin for FanartTvPlugin {
    fn display_name(&self) -> String {
        "fanart.tv".to_string()
    }
    
    fn type_name(&self) -> String {
        "FanartTvPlugin".to_string()
    }

    fn image(&self, host: &Plugins, _library: &Library, model: &dyn Model) -> Result<Option<Dimage>, anyhow::Error> {
        if let Some(artist) = model.as_any().downcast_ref::<Artist>() {
            let mbid = artist.musicbrainz_id.clone().ok_or_else(|| Error::msg("mbid is required"))?;
            let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", mbid, self.api_key);
            let response = host.get(&url)?;
            let artist_resp = response.json::<ArtistResponse>()?;
            let thumb = artist_resp.artistthumb.first().ok_or_else(|| Error::msg("no artistthumbs"))?;
                
            let thumb_resp = host.get(&thumb.url)?;
            let bytes = thumb_resp.bytes()?;
            let image = image::load_from_memory(&bytes)?;
            let mut dimage = Dimage::new(&image);
            dimage.kind = Some(DimageKind::MusicArtistThumb);
            return Ok(Some(dimage))
        }
        Ok(None)
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

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::Artist, plugins::{plugin::Plugin, plugins::Plugins}};

    use super::FanartTvPlugin;

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = FanartTvPlugin::default();
        let image = plugin.image(&plugins, &library, &Artist {
            musicbrainz_id: Some("6821bf3f-5d5b-4b0f-8fa4-79d2ab2d9219".to_string()),
            ..Default::default()
        }).unwrap().unwrap();
        assert!(image.width == 1000);
    }
}