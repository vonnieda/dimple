use dimple_core::collection::{Collection, Model, LibrarySupport};
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
                let request_token = LibrarySupport::start_request(self, &url);
                let client = Client::builder()
                    .user_agent(dimple_core::USER_AGENT)
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

impl Collection for CoverArtArchiveLibrary {
    fn name(&self) -> String {
        "CoverArtArchive".to_string()
    }

    fn image(&self, _entity: &Model) -> Option<image::DynamicImage> {
        match _entity {
            Model::ReleaseGroup(r) => {
                let request_token = LibrarySupport::start_request(self, 
                    &format!("http://coverartarchive.org/{}", r.id));                
                let mb = ReleaseGroup {
                    id: r.id.to_string(),
                    ..Default::default()
                };
                // TODO replace with reqwest
                mb.get_coverart()
                    .front()
                    .res_1200()
                    .execute()
                    .ok()
                    .map(|resp| self.get_coverart(resp))
                    .inspect(|_f| {
                        // TODO
                        LibrarySupport::end_request(request_token, None, None);
                    })
                    ?
            },
            Model::Release(r) => {
                let request_token = LibrarySupport::start_request(self, 
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
                    .res_1200()
                    .execute()
                    .ok()
                    .map(|resp| self.get_coverart(resp))
                    .inspect(|_f| {
                        // TODO
                        LibrarySupport::end_request(request_token, None, None);
                    })
                    ?
            },
            _ => None,
        }
    }
}

