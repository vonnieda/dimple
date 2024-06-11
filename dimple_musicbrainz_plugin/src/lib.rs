use std::sync::Mutex;
use std::time::{Instant, Duration};

use anyhow::{Error, Result};
use client::{ArtistResults, ReleaseGroupResults, ReleaseGroups, Releases};
use converters::{ArtistConverter, RecordingConverter, ReleaseConverter, ReleaseGroupConverter};
use dimple_core::model::{Artist, Entity, Model, Recording, Release, ReleaseGroup};
use dimple_librarian::plugin::{PluginSupport, NetworkMode, Plugin};

mod converters;
mod client;

// https://musicbrainz.org/doc/MusicBrainz_API
// https://musicbrainz.org/doc/MusicBrainz_Entity

#[derive(Debug)]
pub struct MusicBrainzPlugin {
    rate_limit_lock: Mutex<Instant>,
}

impl Default for MusicBrainzPlugin {
    fn default() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_librarian::plugin::USER_AGENT);
        Self {
            rate_limit_lock: Mutex::new(Instant::now()),
        }
    }
}

impl MusicBrainzPlugin {
    /// Blocks until at least one second has passed since the last request.
    fn enforce_rate_limit(&self) {
        let mut last_request_time = self.rate_limit_lock.lock().unwrap();

        if let Some(time_passed) = Instant::now().checked_duration_since(*last_request_time) {
            if time_passed < Duration::from_secs(1) {
                let sleep_duration = Duration::from_secs(1) - time_passed;
                std::thread::sleep(sleep_duration);
            }
        }

        // Update the last request time
        *last_request_time = Instant::now();
    }
}

impl Plugin for MusicBrainzPlugin {
    fn name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn get(&self, model: &Model, network_mode: NetworkMode) -> Result<Option<Model>> {
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }

        match model {
            Model::Artist(artist) => {
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;                
                let url = format!("https://musicbrainz.org/ws/2/artist/{}?fmt=json&inc=aliases annotation genres ratings tags url-rels", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }
                let artist = response.json::<musicbrainz_rs::entity::artist::Artist>()?;
                let artist = Artist::from(ArtistConverter::from(artist.clone()));
                Ok(Some(artist.model()))
            },

            Model::ReleaseGroup(r) => {
                let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
                let url = format!("https://musicbrainz.org/ws/2/release-group/{}?fmt=json&inc=aliases annotation artists genres ratings tags url-rels", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }        
                let release_group = response.json::<musicbrainz_rs::entity::release_group::ReleaseGroup>()?;
                let release_group = ReleaseGroup::from(ReleaseGroupConverter::from(release_group.clone()));
                Ok(Some(release_group.model()))
            },

            Model::Release(r) => {
                let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
                let url = format!("https://musicbrainz.org/ws/2/release/{}?fmt=json&inc=aliases annotation artists genres ratings recordings release-groups tags url-rels", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }        
                let release = response.json::<musicbrainz_rs::entity::release::Release>()?;
                let release = Release::from(ReleaseConverter::from(release.clone()));
                Ok(Some(release.model()))
            },

            Model::Recording(r) => {
                let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
                let url = format!("https://musicbrainz.org/ws/2/recording/{}?fmt=json&inc=aliases annotation artists genres isrcs ratings releases release-groups tags url-rels", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }        
                let recording = response.json::<musicbrainz_rs::entity::recording::Recording>()?;
                let recording = Recording::from(RecordingConverter::from(recording.clone()));
                Ok(Some(recording.model()))
            },

            _ => Ok(None),
        }
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
        network_mode: NetworkMode,
    ) -> Result<Box<dyn Iterator<Item = Model>>> {
        // TODO handle paging
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }
        match (list_of, related_to) {
            (Model::ReleaseGroup(_), Some(Model::Artist(artist))) => {                
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
                let url = format!("https://musicbrainz.org/ws/2/release-group?fmt=json&offset=0&limit=100&artist={}&inc=artist-credits", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }        
                let release_groups = response.json::<ReleaseGroups>()?;
                let iter = release_groups.release_groups.into_iter()
                    .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
                    .map(|src| src.model());
                Ok(Box::new(iter))
            },
            (Model::Release(_), Some(Model::Artist(artist))) => {                
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
                let url = format!("https://musicbrainz.org/ws/2/release?fmt=json&offset=0&limit=100&artist={}&inc=artist-credits labels recordings release-groups media discids isrcs", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }
                let releases = response.json::<Releases>()?;
                let iter = releases.releases.into_iter()
                    .map(|src| Release::from(ReleaseConverter::from(src.clone())))
                    .map(|src| src.model());
                Ok(Box::new(iter))
            },
            (Model::Release(_), Some(Model::ReleaseGroup(release_group))) => {                
                let mbid = release_group.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
                let url = format!("https://musicbrainz.org/ws/2/release?fmt=json&offset=0&limit=100&release-group={}&inc=artist-credits labels recordings release-groups media discids isrcs", mbid);
                let response = PluginSupport::get(self, &url)?;
                if !response.cached() {
                    self.enforce_rate_limit();
                }
                let releases = response.json::<Releases>()?;
                let iter = releases.releases.into_iter()
                    .map(|src| Release::from(ReleaseConverter::from(src.clone())))
                    .map(|src| src.model());
                Ok(Box::new(iter))
            },
            _ => Err(Error::msg("Not implemented.")),
        }        
    }

    // TODO I want to return scores, or at least filter by them.
    fn search(&self, query: &str, network_mode: dimple_librarian::plugin::NetworkMode) 
        -> Result<Box<dyn Iterator<Item = Model>>> {
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }
    
        let iter = std::iter::empty();

        let url = format!("https://musicbrainz.org/ws/2/artist/?query={}&fmt=json", &query);
        let response = PluginSupport::get(self, &url).unwrap();
        if !response.cached() {
            self.enforce_rate_limit();
        }
        let results = response.json::<ArtistResults>().unwrap();
        let models = results.artists.into_iter()
            .map(|src| Artist::from(ArtistConverter::from(src.clone())))
            .map(|src| src.model());
        let iter = iter.chain(models);

        let url = format!("https://musicbrainz.org/ws/2/release-group/?query={}&fmt=json", &query);
        if !response.cached() {
            self.enforce_rate_limit();
        }
        let response = PluginSupport::get(self, &url)?;
        let results = response.json::<ReleaseGroupResults>()?;
        let models = results.release_groups.into_iter()
            .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
            .map(|src| src.model());
        let iter = iter.chain(models);

        Ok(Box::new(iter))
    }    
}

#[cfg(test)]
mod tests {
    use dimple_core::model::{Artist, Entity, KnownIds, Model, Recording, Release, ReleaseGroup};
    use dimple_librarian::plugin::{NetworkMode, Plugin};

    use crate::MusicBrainzPlugin;

    #[test]
    fn get_artist() {
        let plugin = MusicBrainzPlugin::default();
        let artist = Artist {
            known_ids: KnownIds {
                musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let artist = plugin.get(&artist.model(), NetworkMode::Online).unwrap().unwrap();
        println!("{:?}", artist);
    }

    #[test]
    fn get_release_group() {
        let plugin = MusicBrainzPlugin::default();
        let release_group = ReleaseGroup {
            known_ids: KnownIds {
                musicbrainz_id: Some("a96550cd-c202-326d-9593-313f72399ad5".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let release_group = plugin.get(&release_group.model(), NetworkMode::Online).unwrap().unwrap();
        println!("{:?}", release_group);
    }

    #[test]
    fn get_release() {
        let plugin = MusicBrainzPlugin::default();
        let release = Release {
            known_ids: KnownIds {
                musicbrainz_id: Some("006cb56d-6eff-4b7d-853f-ecd2db97f3b2".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let release = plugin.get(&release.model(), NetworkMode::Online).unwrap().unwrap();
        println!("{:#?}", release);
    }

    #[test]
    fn get_recording() {
        let plugin = MusicBrainzPlugin::default();
        let recording = Recording {
            known_ids: KnownIds {
                musicbrainz_id: Some("70ac6733-068f-4613-b06b-bea17cfbcc30".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let recording = plugin.get(&recording.model(), NetworkMode::Online).unwrap().unwrap();
        println!("{:#?}", recording);
    }

    #[test]
    fn list_artist_releases() {
        let plugin = MusicBrainzPlugin::default();
        let artist = Artist {
            known_ids: KnownIds {
                musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let releases: Vec<Release> = plugin.list(&Release::default().model(), 
            &Some(artist.model()), NetworkMode::Online)
            .unwrap()
            .map(|model| Release::from(model))
            .collect();
        println!("{:#?}", releases);
    }

    #[test]
    fn search() {
        let plugin = MusicBrainzPlugin::default();
        let results: Vec<Model> = plugin.search("Nirvana", NetworkMode::Online).unwrap().collect();
        println!("{:#?}", results);
    }
}
















// fn list(
//     &self,
//     list_of: &dyn Entity,
//     related_to: Option<&dyn Entity>,
//     network_mode: dimple_librarian::plugin::NetworkMode,
// ) -> Result<Box<dyn Iterator<Item = Box<dyn Entity>>>> {
//     match (list_of.model(), related_to.map(|related_to| model()) {
//         (Model::ReleaseGroup(_), Some(Entities::Artist(a))) => {                
//             // // TODO handle paging
//             let mbid = a.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/release-group/TODO TODO{}?fmt=json", mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBReleaseGroup::browse().by_artist(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                 .map(Entities::ReleaseGroup)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         (Entities::Release(_), Some(Entities::Artist(a))) => {                
//             // TODO handle paging
//             let mbid = a.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             // https://musicbrainz.org/artist/65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab/releases
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/release?artist={}&fmt=json", &mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBRelease::browse().by_artist(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                 .map(Entities::Release)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         (Entities::Recording(_), Some(Entities::Release(a))) => {
//             // TODO handle paging
//             let mbid = a.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/recording/TODO TODO{}?fmt=json", &mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBRecording::browse().by_release(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//                 .map(Entities::Recording)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         (Entities::Release(_), Some(Entities::ReleaseGroup(r))) => {
//             // TODO handle paging
//             let mbid = r.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/release/TODO TODO{}?fmt=json", &mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBRelease::browse().by_release_group(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                 .map(Entities::Release)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         _ => Box::new(vec![].into_iter()),
//     }
// }
    


// impl Collection for MusicBrainzLibrary {
//     fn name(&self) -> String {
//         "MusicBrainz".to_string()
//     }

//     fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
//         let query = query.to_string();

//         self.enforce_rate_limit();
//         let request_token = LibrarySupport::start_request(self, 
//             &format!("http://musicbrainz.org/search/artist/{}", &query));

//         // TODO And releases, tracks, etc.
//         let search_query = ArtistSearchQuery::query_builder()
//             .artist(&query)
//             .build();
//         let results: Vec<Entities> = MBArtist::search(search_query)
//             .execute()
//             .inspect(|_f| {
//                 LibrarySupport::end_request(request_token, None, None);
//             })
//             // TODO
//             .unwrap()
//             .entities
//             .iter()
//             .map(|src| dimple_core::model::Artist::from(ArtistConverter::from(src.clone())))
//             .map(Entities::Artist)
//             .collect();
//         Box::new(results.into_iter())
//     }

//     fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
//         match (of_type, related_to) {
//             (Entities::ReleaseGroup(_), Some(Entities::Artist(a))) => {                
//                 // // TODO handle paging
//                 let mbid = a.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release-group/TODO TODO{}?fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBReleaseGroup::browse().by_artist(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                     .map(Entities::ReleaseGroup)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             (Entities::Release(_), Some(Entities::Artist(a))) => {                
//                 // TODO handle paging
//                 let mbid = a.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 // https://musicbrainz.org/artist/65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab/releases
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release?artist={}&fmt=json", &mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBRelease::browse().by_artist(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(Entities::Release)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             (Entities::Recording(_), Some(Entities::Release(a))) => {
//                 // TODO handle paging
//                 let mbid = a.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/recording/TODO TODO{}?fmt=json", &mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBRecording::browse().by_release(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//                     .map(Entities::Recording)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             (Entities::Release(_), Some(Entities::ReleaseGroup(r))) => {
//                 // TODO handle paging
//                 let mbid = r.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release/TODO TODO{}?fmt=json", &mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBRelease::browse().by_release_group(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(Entities::Release)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             _ => Box::new(vec![].into_iter()),
//         }
//     }

//     fn fetch(&self, entity: &Entities) -> Option<Entities> {
//         match entity {
//             Entities::Artist(a) => {
//                 let mbid = a.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/artist/{}?inc=aliases%20release-groups%20releases%20release-group-rels%20release-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBArtist::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_genres().with_rating()
//                     .with_tags().with_release_groups().with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| Artist::from(ArtistConverter::from(src.clone())))
//                     .map(Entities::Artist)        
//             },
//             Entities::ReleaseGroup(r) => {
//                 let mbid = r.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release-group/{}?inc=aliases%20artists%20releases%20release-group-rels%20release-rels%20url-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBReleaseGroup::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_artists()
//                     .with_genres().with_ratings().with_releases().with_tags()
//                     .with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                     .map(Entities::ReleaseGroup)        
//             },
//             Entities::Release(r) => {
//                 let mbid = r.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release/{}?inc=aliases%20artist-credits%20artist-rels%20artists%20genres%20labels%20ratings%20recording-rels%20recordings%20release-groups%20release-group-rels%20tags%20release-rels%20url-rels%20work-level-rels%20work-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBRelease::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_artist_credits()
//                     .with_artists().with_genres().with_labels().with_ratings()
//                     .with_recordings().with_release_groups().with_tags()
//                     .with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(Entities::Release)        
//             },
//             Entities::Recording(r) => {
//                 let mbid = r.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/recording/{}?inc=aliases%20artist-credits%20artist-rels%20artists%20genres%20labels%20ratings%20recording-rels%20recordings%20release-groups%20release-group-rels%20tags%20release-rels%20url-rels%20work-level-rels%20work-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBRecording::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_artists()
//                     .with_genres().with_isrcs().with_ratings().with_releases()
//                     .with_tags().with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//                     .map(Entities::Recording)        
//             },
//             Entities::Genre(_) => None,
//             Entities::RecordingSource(_) => None,
//             Entities::MediaFile(_) => None,
//             Entities::Track(_) => None,
//             Entities::Medium(_) => None,
//         }        
//     }
// }
