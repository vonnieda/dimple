use dimple_core::library::{Library, LibraryEntity, LibrarySupport};
use image::DynamicImage;
use musicbrainz_rs::entity::{CoverartResponse, release_group::ReleaseGroup, release::Release};
use musicbrainz_rs::FetchCoverart;
use reqwest::blocking::Client;

#[derive(Debug, Default)]
pub struct CoverArtArchiveLibrary {
}

impl CoverArtArchiveLibrary {
    pub fn new() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_core::USER_AGENT);
        Self {
        }
    }

    pub fn get_coverart(&self, resp: CoverartResponse) -> Option<DynamicImage> {
        match resp {
            musicbrainz_rs::entity::CoverartResponse::Json(_) => todo!(),
            musicbrainz_rs::entity::CoverartResponse::Url(url) => {
                LibrarySupport::log_request(self, &url);
                let client = Client::builder()
                    .user_agent(dimple_core::USER_AGENT)
                    .build().unwrap();
                client.get(url).send().ok()
                    .map(|resp| resp.bytes().ok())?
                    .and_then(|bytes| image::load_from_memory(&bytes).ok())
            },    
        }
    }
}

impl Library for CoverArtArchiveLibrary {
    fn name(&self) -> String {
        "CoverArtArchive".to_string()
    }

    fn image(&self, _entity: &LibraryEntity) -> Option<image::DynamicImage> {
        match _entity {
            LibraryEntity::ReleaseGroup(r) => {
                LibrarySupport::log_request(self, 
                    &format!("http://coverartarchive.org/{}", r.id));                
                let mb = ReleaseGroup {
                    id: r.id.to_string(),
                    ..Default::default()
                };
                mb.get_coverart()
                    .front()
                    .execute()
                    .ok()
                    .map(|resp| self.get_coverart(resp))?
            },
            LibraryEntity::Release(r) => {
                LibrarySupport::log_request(self, 
                    &format!("http://coverartarchive.org/{}", r.id));                
                let mb = Release {
                    id: r.id.to_string(),
                    title: "".to_string(),
                    aliases: None,
                    annotation:  None,
                    artist_credit: None,
                    barcode: None,
                    country: None,
                    date: None,
                    disambiguation: None,
                    genres: None,
                    label_info: None,
                    status_id: None,
                    status: None,
                    quality: None,
                    packaging_id: None,
                    packaging: None,
                    relations: None,
                    release_group: None,
                    media: None,
                    tags: None,
                };
                mb.get_coverart()
                    .front()
                    .execute()
                    .ok()
                    .map(|resp| self.get_coverart(resp))?
            },
            _ => None,
        }
    }
}

