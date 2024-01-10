use dimple_core::library::{Library, LibraryEntity, LibrarySupport};
use image::DynamicImage;
use musicbrainz_rs::entity::CoverartResponse;
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::{prelude::*, FetchQuery};

#[derive(Debug, Default)]
pub struct MusicBrainzLibrary {
}

impl MusicBrainzLibrary {
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
                reqwest::blocking::get(url).ok()
                    .map(|resp| resp.bytes().ok())?
                    .and_then(|bytes| image::load_from_memory(&bytes).ok())
            },    
        }
    }
}

// TODO all of the log_requests below are semi made up cause I can't get the
// real URL from the FetchQuery etc.
impl Library for MusicBrainzLibrary {
    fn name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
        let query = query.to_string();

        LibrarySupport::log_request(self, 
            &format!("http://musicbrainz.org/search/artist/{}", &query));

        // TODO And releases, tracks, etc.
        let search_query = ArtistSearchQuery::query_builder()
                .artist(&query)
                .build();
        let results: Vec<LibraryEntity> = Artist::search(search_query)
            .execute().unwrap() // TODO error handling
            .entities
            .iter()
            .map(|src| {
                dimple_core::model::Artist {
                    mb: src.clone()
                }
            })
            .map(LibraryEntity::Artist)
            .collect();
        Box::new(results.into_iter())
    }

    fn fetch(&self, _entity: &LibraryEntity) -> Option<LibraryEntity> {
        match _entity {
            LibraryEntity::Artist(a) => {
                LibrarySupport::log_request(self, 
                    &format!("http://musicbrainz.org/fetch/artist/{}", a.mbid()));
                Artist::fetch()
                    .id(&a.mbid())
                    .with_aliases()
                    .with_annotations()
                    .with_genres()
                    .with_rating()
                    .with_tags()
                    .with_releases()
                    .with_release_groups()
                    .with_url_relations()
                    .execute()
                    .ok()
                    .map(|src| {
                        dimple_core::model::Artist {
                            mb: src.clone()
                        }
                    })
                    .map(LibraryEntity::Artist)
                },
            LibraryEntity::Genre(_) => None,
            LibraryEntity::Release(_) => None,
            LibraryEntity::Track(_) => None,
        }        
    }

    fn image(&self, _entity: &LibraryEntity) -> Option<image::DynamicImage> {
        match _entity {
            LibraryEntity::Release(r) => {
                LibrarySupport::log_request(self, 
                    &format!("http://coverartarchive.org/{}", r.mbid()));
                r.mb.get_coverart()
                    .front()
                    .res_250()
                    .execute()
                    .ok()
                    .map(|resp| self.get_coverart(resp))?
            },
            LibraryEntity::Artist(_) => None,
            LibraryEntity::Genre(_) => None,
            LibraryEntity::Track(_) => None,
            
        }
    }
}