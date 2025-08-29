
use anyhow::Error;
use serde::Deserialize;

use crate::{model::{Dimage, DimpleEntity}, plugins::plugin::Plugin};

pub const FANART_TV_API_KEY: &str ="523532";

#[derive(Debug)]
pub struct TheAudioDbPlugin {
    api_key: String,
}

impl Default for TheAudioDbPlugin {
    fn default() -> Self {
        Self::new(FANART_TV_API_KEY)
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
    fn display_name(&self) -> String {
        "The Audio DB".to_string()
    }
    
    fn type_name(&self) -> String {
        "TheAudioDbPlugin".to_string()
    }
    
    fn image(&self, plugins: &super::plugins::Plugins, _library: &crate::library::Library, model: &crate::model::DimpleEntity) -> Result<Option<crate::model::Dimage>, anyhow::Error> {
        match model {
            DimpleEntity::Artist(artist) => {
                let mbid = artist.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;

                let url = format!("https://www.theaudiodb.com/api/v1/json/{}/artist-mb.php?i={}", 
                    self.api_key, mbid);
                let response = plugins.get(&url)?;
                let artists_resp = response.json::<ArtistsResponse>()?;

                let artist_thumbnail_url = artists_resp.artists.first().ok_or(Error::msg("no thumbnail"))?
                    .strArtistThumb.clone();
                if artist_thumbnail_url.is_empty() {
                    return Ok(None)
                }

                let thumb_resp = plugins.get(&artist_thumbnail_url)?;
                let bytes = thumb_resp.bytes()?;
                let image = image::load_from_memory(&bytes)?;

                let mut dimage = Dimage::default();
                dimage.set_image(&image);
                Ok(Some(dimage))
            },
            _ => Ok(None),
        }
    }
}