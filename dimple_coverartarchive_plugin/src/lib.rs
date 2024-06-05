use std::collections::HashSet;

use anyhow::{Error, Result};
use dimple_core::model::{Entity, Model, Picture};
use dimple_librarian::plugin::{LibrarySupport, NetworkMode, Plugin};
use image::DynamicImage;
use musicbrainz_rs::entity::{CoverartResponse, release_group::ReleaseGroup, release::Release};
use musicbrainz_rs::FetchCoverart;
use reqwest::blocking::Client;

#[derive(Debug, Default)]
pub struct CoverArtArchivePlugin {
}

impl CoverArtArchivePlugin {
    pub fn new() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_librarian::plugin::USER_AGENT);
        Self {
        }
    }

    pub fn get_coverart(&self, resp: CoverartResponse) -> Option<DynamicImage> {
        match resp {
            musicbrainz_rs::entity::CoverartResponse::Json(_) => todo!(),
            musicbrainz_rs::entity::CoverartResponse::Url(url) => {
                let request_token = LibrarySupport::start_request(self, &url);
                let client = Client::builder()
                    .user_agent(dimple_librarian::plugin::USER_AGENT)
                    .build().unwrap();
                let response = client.get(&url).send().ok()?;
                let status = response.status();
                let content_length = response.content_length();
                let bytes = response.bytes().ok()?;
                LibrarySupport::end_request(request_token, 
                    Some(status.as_u16()), 
                    content_length);
                let image = image::load_from_memory(&bytes).ok()?;
                Some(image)
            },    
        }
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
            (Model::Picture(_), Some(Model::ReleaseGroup(rg))) => {
                let mbid = rg.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;

                let request_token = LibrarySupport::start_request(self, 
                    &format!("http://coverartarchive.org/release-group/{}", mbid));
                let mb = ReleaseGroup {
                    id: mbid,
                    ..Default::default()
                };
                // TODO replace with reqwest
                let response = mb.get_coverart()
                    .front()
                    // .res_1200()
                    .execute()?;
                let image = self.get_coverart(response).ok_or(Error::msg("download failed"))?;
                LibrarySupport::end_request(request_token, None, None);

                let mut picture = Picture::default();
                picture.set_image(&image);
                Ok(Box::new(std::iter::once(picture.model())))
            },
            _ => Ok(Box::new(std::iter::empty())),
        }
    }

    

    // fn image(&self, _entity: &Entities) -> Option<image::DynamicImage> {
    //     match _entity {
    //         Entities::ReleaseGroup(r) => {
    //             let mbid = r.mbid()?;
    //             let request_token = LibrarySupport::start_request(self, 
    //                 &format!("http://coverartarchive.org/{}", mbid));
    //             let mb = ReleaseGroup {
    //                 id: mbid,
    //                 ..Default::default()
    //             };
    //             // TODO replace with reqwest
    //             mb.get_coverart()
    //                 .front()
    //                 .res_1200()
    //                 .execute()
    //                 .ok()
    //                 .map(|resp| self.get_coverart(resp))
    //                 .inspect(|_f| {
    //                     // TODO
    //                     LibrarySupport::end_request(request_token, None, None);
    //                 })
    //                 ?
    //         },
    //         Entities::Release(r) => {
    //             let mbid = r.mbid()?;
    //             let request_token = LibrarySupport::start_request(self, 
    //                 &format!("http://coverartarchive.org/{}", mbid));                
    //             let mb = Release {
    //                 id: mbid,
    //                 title: "".to_string(),
    //                 aliases: None,
    //                 annotation:  None,
    //                 artist_credit: None,
    //                 barcode: None,
    //                 country: None,
    //                 date: None,
    //                 disambiguation: None,
    //                 genres: None,
    //                 label_info: None,
    //                 status_id: None,
    //                 status: None,
    //                 quality: None,
    //                 packaging_id: None,
    //                 packaging: None,
    //                 relations: None,
    //                 release_group: None,
    //                 media: None,
    //                 tags: None,
    //             };
    //             mb.get_coverart()
    //                 .front()
    //                 .res_1200()
    //                 .execute()
    //                 .ok()
    //                 .map(|resp| self.get_coverart(resp))
    //                 .inspect(|_f| {
    //                     // TODO
    //                     LibrarySupport::end_request(request_token, None, None);
    //                 })
    //                 ?
    //         },
    //         _ => None,
    //     }
    // }
}

