// Lyrics via https://lrclib.net/docs

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{library::Library, model::Track};

use super::{Plugin, USER_AGENT};

#[derive(Default)]
pub struct LrclibPlugin {
    config: LrclibPluginConfig,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct LrclibPluginConfig {
    
}

impl Plugin for LrclibPlugin {
    fn type_name(&self) -> String {
        "LrclibPlugin".to_string()
    }

    fn display_name(&self) -> String {
        "LRCLIB".to_string()
    }

    fn set_configuration(&mut self, config: &str) {
        self.config = serde_json::from_str(config).unwrap();
    }

    fn configuration(&self) -> String {
        serde_json::to_string(&self.config).unwrap()
    }

    fn status(&self) -> String {
        "Ready".to_string()
    }

    fn lyrics(&self, _library: &Library, track: &Track) 
            -> Option<String> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}",
            track.artist.clone().unwrap_or_default(),
            track.title.clone().unwrap_or_default(),
            track.album.clone().unwrap_or_default(),
        );
        let response = client.get(&url).send().unwrap();
        log::info!("GET {} {}", response.status().as_u16(),& url);
        if response.status() != 200 {
            return None
        }
        let get_response = response.json::<GetResponse>().unwrap();
        get_response.plain_lyrics
    }
}

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
    use crate::{library::Library, model::Track, plugins::Plugin};

    use super::LrclibPlugin;

    #[test]
    fn it_works() {
        let library = Library::open("file:d804a172-71cf-4522-a3be-3ce0de93481a?mode=memory&cache=shared");
        let track = library.save(&Track {
            artist: Some("Metallica".to_string()),
            album: Some("Master of Puppets".to_string()),
            title: Some("Master of Puppets".to_string()),
            ..Default::default()
        });
        let lrclib = LrclibPlugin::default();
        let lyrics = lrclib.lyrics(&library, &track);
        assert!(lyrics.is_some());
        assert!(lyrics.unwrap().contains("strings"));
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


// let client = Client::builder()
// .user_agent(super::plugin::USER_AGENT)
// .build()?;
// let request_token = PluginSupport::start_request(plugin, &url);
// let response = client.get(url).send()?;
// PluginSupport::end_request(request_token, 
// Some(response.status().as_u16()), 
// response.content_length());
// let bytes = response.bytes()?;
// if let Some(cache_path) = self.cache_path.clone() {
// cacache::write_sync(cache_path.clone(), url, &bytes)?;
// }
// return Ok(CacheResponse::new(bytes.to_vec(), false))
