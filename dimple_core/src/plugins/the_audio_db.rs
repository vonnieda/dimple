
use serde::Deserialize;

use crate::plugins::plugin::Plugin;

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
    
    fn image(&self, _host: &super::plugins::Plugins, _library: &crate::library::Library, _model: &crate::model::DimpleEntity) -> Result<Option<crate::model::Dimage>, anyhow::Error> {
        Ok(None)
    }


    // fn list(
    //     &self,
    //     list_of: &dimple_core::model::Model,
    //     related_to: &Option<dimple_core::model::Model>,
    //     network_mode: dimple_librarian::plugin::NetworkMode,
    //     ctx: &PluginContext,
    // ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
    //     if network_mode != NetworkMode::Online {
    //         return Err(Error::msg("Offline."))
    //     }

    //     match (list_of, related_to) {
    //         (Model::Dimage(_), Some(Model::Artist(artist))) => {
    //             let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;

    //             let url = format!("https://www.theaudiodb.com/api/v1/json/{}/artist-mb.php?i={}", 
    //                 self.api_key, mbid);
    //             let response = ctx.get(self, &url)?;
    //             let artists_resp = response.json::<ArtistsResponse>()?;

    //             let artist_thumbnail_url = artists_resp.artists.first().ok_or(Error::msg("no thumbnail"))?
    //                 .strArtistThumb.clone();
    //             if artist_thumbnail_url.is_empty() {
    //                 return Ok(Box::new(std::iter::empty()))
    //             }

    //             let thumb_resp = ctx.get(self, &artist_thumbnail_url)?;
    //             let bytes = thumb_resp.bytes()?;
    //             let image = image::load_from_memory(&bytes)?;

    //             let mut dimage = Dimage::default();
    //             dimage.set_image(&image);
    //             Ok(Box::new(std::iter::once(dimage.model())))
    //         },
    //         _ => Ok(Box::new(std::iter::empty())),
    //     }
    // }
}