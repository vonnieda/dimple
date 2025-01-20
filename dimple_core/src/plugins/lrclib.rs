// Lyrics via https://lrclib.net/docs

use anyhow::anyhow;
use reqwest::blocking::Client;
use serde::{Deserialize};

use crate::{library::Library, model::{Model, Track}};

use super::{plugin::{Plugin}, plugin_host::PluginHost, USER_AGENT};

#[derive(Default)]
pub struct LrclibPlugin {
}

impl Plugin for LrclibPlugin {
    fn type_name(&self) -> String {
        "LrclibPlugin".to_string()
    }

    fn display_name(&self) -> String {
        "LRCLIB".to_string()
    }

    // TODO start with album + artist and fallback to no album
    // let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}",
    //     track.artist.clone().unwrap_or_default(),
    //     track.title.clone().unwrap_or_default(),
    //     track.album.clone().unwrap_or_default(),
    // );
    fn metadata(&self, host: &PluginHost, library: &Library, model: &dyn Model) 
        -> Result<Option<Box<dyn Model>>, anyhow::Error> {

        if let Some(track) = model.as_any().downcast_ref::<Track>() {
            let artist = track.artist(library).ok_or(anyhow!("artist is required"))?;
            let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}",
                artist.name.clone().unwrap_or_default(),
                track.title.clone().unwrap_or_default(),
            );
            let response: GetResponse = host.get(&url)?.json()?;
            return Ok(Some(Box::new(Track {
                lyrics: response.plain_lyrics,
                synchronized_lyrics: response.synced_lyrics,
                title: response.track_name,
                length_ms: response.duration_s.map(|s| (s * 1000.) as u64),
                ..Default::default()
            })))
        }

        Ok(None)
    }    
}

// {
//     "id": 3396226,
//     "trackName": "I Want to Live",
//     "artistName": "Borislav Slavov",
//     "albumName": "Baldur's Gate 3 (Original Game Soundtrack)",
//     "duration": 233,
//     "instrumental": false,
//     "plainLyrics": "I feel your breath upon my neck\n...The clock won't stop and this is what we get\n",
//     "syncedLyrics": "[00:17.12] I feel your breath upon my neck\n...[03:20.31] The clock won't stop and this is what we get\n[03:25.72] "
//   }
#[derive(Clone, Debug, Deserialize)]
struct GetResponse {
    pub id: Option<f64>,
    #[serde(rename(serialize = "trackName", deserialize = "trackName"))]
    pub track_name: Option<String>,
    #[serde(rename(serialize = "artistName", deserialize = "artistName"))]
    pub artist_name: Option<String>,
    #[serde(rename(serialize = "albumName", deserialize = "albumName"))]
    pub album_name: Option<String>,
    #[serde(rename(serialize = "duration", deserialize = "duration"))]
    pub duration_s: Option<f64>,
    pub instrumental: Option<bool>,
    #[serde(rename(serialize = "plainLyrics", deserialize = "plainLyrics"))]
    pub plain_lyrics: Option<String>,
    #[serde(rename(serialize = "syncedLyrics", deserialize = "syncedLyrics"))]
    pub synced_lyrics: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Track}, plugins::{plugin::Plugin, plugin_host::PluginHost}};

    use super::LrclibPlugin;

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();

        let library = Library::open_memory();

        let artist = library.save(&Artist {
            name: Some("Metallica".to_string()),
            ..Default::default()
        });
        let track = library.save(&Track {
            title: Some("Master of Puppets".to_string()),
            ..Default::default()
        });
        let _ = library.save(&ArtistRef {
            model_key: track.key.clone().unwrap(),
            artist_key: artist.key.clone().unwrap(),
            ..Default::default()
        });

        let lrclib = LrclibPlugin::default();
        let host = PluginHost::default();
        let track = lrclib.metadata(&host, &library, &track).unwrap().unwrap();
        let track = track.as_any().downcast_ref::<Track>().unwrap().clone();
        assert!(track.lyrics.unwrap()
            .to_lowercase().contains("pulling your strings"));
    }
}
