use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::{library::Library, model::{dimage::DimageKind, Dimage, DimpleEntity}, plugins::{plugin::Plugin, plugins::Plugins}};

#[derive(Debug, Default)]
pub struct CoverArtArchivePlugin {
}

impl CoverArtArchivePlugin {
    fn get_coverart(&self, url: &str, plugins: &Plugins) -> Result<Option<Dimage>> {
        let response: CoverArtResponse = plugins.get(url)?.json()?;
        for image in response.images {
            if image.approved && image.front {
                let image_response = plugins.get(&image.image)?;
                let bytes = image_response.bytes()?;
                let image = image::load_from_memory(&bytes)?;
                let mut dimage = Dimage::new(&image);
                dimage.kind = Some(DimageKind::MusicArtistThumb);
                return Ok(Some(dimage))
            }
        }
        Ok(None)
    }
}

impl Plugin for CoverArtArchivePlugin {
    fn type_name(&self) -> String {
        "CoverArtArchivePlugin".to_string()
    }

    fn display_name(&self) -> String {
        "Cover Art Archive".to_string()
    }
    
    fn image(&self, host: &Plugins, _library: &Library, model: &DimpleEntity) -> Result<Option<Dimage>, anyhow::Error> {
        match model {
            DimpleEntity::Release(release) => {
                let mbid = release.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
                let url = format!("http://coverartarchive.org/release/{mbid}");
                let dimage = self.get_coverart(&url, host)?;
                Ok(dimage)
            }
            _ => Ok(None)
        }
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct CoverArtResponse {
    images: Vec<CoverArtImage>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct CoverArtImage {
    image: String,
    front: bool,
    back: bool,
    approved: bool,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Release}, plugins::{coverart_archive::CoverArtArchivePlugin, plugin::Plugin as _, plugins::Plugins}};

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = CoverArtArchivePlugin::default();
        let artist = library.save(&Release {
            musicbrainz_id: Some("76df3287-6cda-33eb-8e9a-044b5e15ffdd".to_string()),
            ..Default::default()
        }).unwrap();
        let image = plugin.image(&plugins, &library, &artist.into()).unwrap().unwrap();
        assert_eq!(image.width, 538);
    }
}
