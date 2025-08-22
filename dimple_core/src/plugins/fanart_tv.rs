use std::collections::HashMap;

use anyhow::{Error, Result};
use serde::Deserialize;

use crate::{library::Library, model::{dimage::DimageKind, Dimage, DimpleEntity}};

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

// Project key for jason@vonnieda.org. Distributing it and letting users
// also add their personal access key seems to be the preferred method as
// evidenced by the source of some other music players.
pub const FANART_TV_API_KEY: &str ="dae13ed416ea0d16994d391db0d7ad3d";

#[derive(Debug)]
pub struct FanartTvPlugin {
    pub api_key: String,
}

impl Default for FanartTvPlugin {
    fn default() -> Self {
        Self::new(FANART_TV_API_KEY)
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

    fn image(&self, host: &Plugins, _library: &Library, model: &DimpleEntity) -> Result<Option<Dimage>, anyhow::Error> {
        match model {
            DimpleEntity::Artist(artist) => {
                let mbid = artist.musicbrainz_id.clone().ok_or_else(|| Error::msg("mbid is required"))?;

                let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", mbid, &self.api_key);
                let response = host.get(&url)?;
                if response.status() == 404 {
                    return Ok(None)
                }
                let artist_resp = response.json::<ArtistResponse>()?;
                let mut it = artist_resp.artistthumb.iter()
                    .chain(artist_resp.hdmusiclogo.iter())
                    .chain(artist_resp.musiclogo.iter())
                    .chain(artist_resp.artistbackground.iter())
                    .chain(artist_resp.albums.values().flat_map(|e| {
                        e.albumcover.iter().chain(e.cdart.iter())
                    }));
                let thumb = it.next().ok_or_else(|| Error::msg("no artist images in response"))?;
                    
                let thumb_resp = host.get(&thumb.url)?;
                let bytes = thumb_resp.bytes()?;
                let image = image::load_from_memory(&bytes)?;
                let mut dimage = Dimage::new(&image);
                // TODO set correct type based on what is found above
                dimage.kind = Some(DimageKind::MusicArtistThumb);
                Ok(Some(dimage))
            },
            _ => Ok(None)
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
    albums: HashMap<String, AlbumResponse>,
    status: String,
    error_message: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct AlbumResponse {
    cdart: Vec<ImageResponse>,
    albumcover: Vec<ImageResponse>,
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
    use crate::{library::Library, model::{Artist}, plugins::{plugin::Plugin, plugins::Plugins}};

    use super::FanartTvPlugin;

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = FanartTvPlugin::default();
        let artist = library.save(&Artist {
            musicbrainz_id: Some("6821bf3f-5d5b-4b0f-8fa4-79d2ab2d9219".to_string()),
            ..Default::default()
        }).unwrap();
        let image = plugin.image(&plugins, &library, &artist.into()).unwrap().unwrap();
        assert!(image.width == 1000);
    }
}