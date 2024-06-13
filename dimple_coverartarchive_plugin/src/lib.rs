use anyhow::{Error, Result};
use dimple_core::model::{Entity, Model, Dimage};
use dimple_librarian::plugin::{PluginSupport, NetworkMode, Plugin};
use image::DynamicImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct CoverArtArchivePlugin {
}

impl CoverArtArchivePlugin {
    fn get_coverart(&self, url: &str) -> Result<DynamicImage> {
        let response: CoverArtResponse = PluginSupport::get(self, url)?.json()?;
        for image in response.images {
            if image.approved && image.front {
                let image_response = PluginSupport::get(self, &image.image)?;
                let bytes = image_response.bytes()?;
                let image = image::load_from_memory(&bytes)?;
                return Ok(image)
            }
        }
        Err(Error::msg("No suitable image found"))
    }
}

impl Plugin for CoverArtArchivePlugin {
    fn name(&self) -> String {
        "CoverArtArchive".to_string()
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
            (Model::Dimage(_), Some(Model::ReleaseGroup(rg))) => {
                let mbid = rg.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
                let url = format!("http://coverartarchive.org/release-group/{}", mbid);
                let image = self.get_coverart(&url)?;
                let mut dimage = Dimage::default();
                dimage.set_image(&image);
                Ok(Box::new(std::iter::once(dimage.model())))
            },
            (Model::Dimage(_), Some(Model::Release(rg))) => {
                let mbid = rg.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
                let url = format!("http://coverartarchive.org/release/{}", mbid);
                let image = self.get_coverart(&url)?;
                let mut dimage = Dimage::default();
                dimage.set_image(&image);
                Ok(Box::new(std::iter::once(dimage.model())))
            },
            _ => Ok(Box::new(std::iter::empty())),
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
    use dimple_core::model::{Entity, KnownIds, Dimage, Release};
    use dimple_librarian::plugin::Plugin;

    use crate::CoverArtArchivePlugin;

    #[test]
    fn basics() {
        let plugin = CoverArtArchivePlugin::default();
        let release = Release {
            known_ids: KnownIds {
                musicbrainz_id: Some("76df3287-6cda-33eb-8e9a-044b5e15ffdd".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let results = plugin.list(&Dimage::default().model(), &Some(release.model()), 
            dimple_librarian::plugin::NetworkMode::Online).unwrap();
        for result in results {
            let dimage: Dimage = result.into();
            let image = dimage.get_image();
            println!("{}x{}", image.width(), image.height());
        }
    }
}
