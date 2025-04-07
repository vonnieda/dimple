use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use serde::{Deserialize, Serialize};

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Model, Release, Track}, plugins::converters::ReleaseConverter};

use super::{converters::{ArtistConverter, TrackConverter}, plugin::Plugin, plugin_host::PluginHost};

pub struct MusicBrainzPlugin {
    config: MusicBrainzPluginConfig,
    rate_limit_lock: Arc<Mutex<Instant>>,
}

impl Default for MusicBrainzPlugin {
    fn default() -> Self {
        Self { 
            config: Default::default(), 
            rate_limit_lock: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

impl Plugin for MusicBrainzPlugin {
    fn type_name(&self) -> String {
        "MusicBrainzPlugin".to_string()
    }

    fn display_name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn set_configuration(&mut self, config: &str) {
        self.config = serde_json::from_str(config).unwrap();
    }

    fn configuration(&self) -> String {
        serde_json::to_string(&self.config).unwrap()
    }

    fn artist_metadata(&self, host: &PluginHost, _library: &Library, artist: &Artist) 
        -> Result<Option<ArtistMetadata>, anyhow::Error> {

        if let Some(mbid) = artist.musicbrainz_id.clone() {
            let url = format!("https://musicbrainz.org/ws/2/artist/{}?fmt=json&inc=aliases+annotation+genres+ratings+tags+url-rels", mbid);
            self.enforce_rate_limit();
            let response = host.get(&url)?;
            let mb_artist = response.json::<musicbrainz_rs::entity::artist::Artist>()?;
            let artist_metadata: ArtistMetadata = ArtistConverter::from(mb_artist).into();
            return Ok(Some(artist_metadata))
        }
        Ok(None)
    }

    fn release_metadata(&self, host: &PluginHost, _library: &Library, release: &Release) 
        -> Result<Option<ReleaseMetadata>, anyhow::Error> {

        if let Some(mbid) = release.musicbrainz_id.clone() {
            let url = format!("https://musicbrainz.org/ws/2/release/{}?fmt=json&inc=aliases+annotation+artists+genres+media+ratings+recordings+release-groups+tags+url-rels", mbid);
            self.enforce_rate_limit();
            let response = host.get(&url)?;
            let mb_release = response.json::<musicbrainz_rs::entity::release::Release>()?;
            let release_metadata: ReleaseMetadata = ReleaseConverter::from(mb_release).into();
            return Ok(Some(release_metadata))
        }
        Ok(None)
    }

    fn track_metadata(&self, host: &PluginHost, _library: &Library, track: &Track) 
        -> Result<Option<TrackMetadata>, anyhow::Error> {
        // if let Some(mbid) = track.musicbrainz_id.clone() {
        //     let url = format!("https://musicbrainz.org/ws/2/artist/{}?fmt=json&inc=aliases+annotation+genres+ratings+tags+url-rels", mbid);
        //     let response = host.get(&url)?;
        //     let mb_track = response.json::<musicbrainz_rs::entity::release::Track>()?;
        //     let track_metadata: TrackMetadata = TrackConverter::from(mb_track).into();
        //     return Ok(Some(track_metadata))
        // }
        Ok(None)
    }
    
    fn search(&self, host: &PluginHost, library: &Library, query: &str) 
        -> Result<crate::librarian::SearchResults, anyhow::Error> {
        
        // http://musicbrainz.org/ws/2/artist/?query=artist:klok
        let url = format!("https://musicbrainz.org/ws/2/artist/?fmt=json&query={}", query);
        self.enforce_rate_limit();
        let response = host.get(&url)?;
        let mb_results = response.json::<musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::artist::Artist>>()?;
        let artists: Vec<ArtistMetadata> = mb_results.entities.into_iter().map(|e| ArtistConverter::from(e).into()).collect();

        let url = format!("https://musicbrainz.org/ws/2/release/?fmt=json&query={}", query);
        if !response.cached() {
            self.enforce_rate_limit();
        }
        let response = host.get(&url)?;
        let mb_results = response.json::<musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release>>()?;
        let releases: Vec<ReleaseMetadata> = mb_results.entities.into_iter().map(|e| ReleaseConverter::from(e).into()).collect();

        Ok(SearchResults {
            artists: artists.into_iter().map(|e| e.artist).collect(),
            releases: releases.into_iter().map(|e| e.release).collect(),
            ..Default::default()
        })
    }

    // fn artist_releases() {
    //     let mut offset: u32 = 0;
    //     let releases: Vec<ReleaseMetadata>
    //     loop {
    //         let url = format!("https://musicbrainz.org/ws/2/release?fmt=json&offset={}&limit=100&artist={}&inc=artist-credits", 
    //             offset, mbid);
    //         if !response.cached() {
    //             self.enforce_rate_limit();
    //         }
    //         let response = host.get(&url)?;
    //         let mb_artist = response.json::<musicbrainz_rs::entity::artist::Artist>()?;
    //         let artist_metadata: ArtistMetadata = ArtistConverter::from(mb_artist).into();
    //     }


    // //                 let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
    // //                 let url = format!("https://musicbrainz.org/ws/2/release-group?fmt=json&offset=0&limit=100&artist={}&inc=artist-credits", mbid);
    // //                 let response = ctx.get(self, &url)?;
    // //                 if !response.cached() {
    // //                     self.enforce_rate_limit();
    // //                 }        
    // //                 let release_groups = response.json::<ReleaseGroups>()?;
    // //                 let iter = release_groups.release_groups.into_iter()
    // //                     .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
    // //                     .map(|src| src.model());
    // //                 Ok(Box::new(iter))
    // }
}

fn nempty(s: String) -> Option<String> {
    if s.is_empty() {
        return None
    }
    Some(s)
}

impl MusicBrainzPlugin {
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

#[derive(Serialize, Deserialize, Clone, Default)]
struct MusicBrainzPluginConfig {    
    pub url: String,
    pub username: String,
    pub password: String,
}

mod tests {
    use crate::{library::Library, model::Artist, plugins::{plugin::Plugin, plugin_host::PluginHost}};

    use super::MusicBrainzPlugin;

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugin_host = PluginHost::default();
        let plugin = MusicBrainzPlugin::default();
        let metadata = plugin.artist_metadata(&plugin_host, &library, &Artist {
            musicbrainz_id: Some("6821bf3f-5d5b-4b0f-8fa4-79d2ab2d9219".to_string()),
            ..Default::default()
        }).unwrap().unwrap();
        assert!(metadata.artist.name == Some("Blonde Redhead".to_string()));
    }

    
    #[test]
    fn search() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugin_host = PluginHost::default();
        let plugin = MusicBrainzPlugin::default();
        let results = plugin.search(&plugin_host, &library, "death clock").unwrap();
        dbg!(results);
    }    
}

// // https://musicbrainz.org/doc/MusicBrainz_Entity
// #[derive(Debug)]
// pub struct MusicBrainzClient {
//     rate_limit_lock: Mutex<Instant>,
// }

// impl MusicBrainzClient {
//     /// Blocks until at least one second has passed since the last request.
//     /// TODO I think I can adjust this to average over 10 seconds or something
//     /// so that we can do quick bursts without feeling slow and without
//     /// passing the rate limit.
//     fn enforce_rate_limit(&self) {
//         let mut last_request_time = self.rate_limit_lock.lock().unwrap();

//         if let Some(time_passed) = Instant::now().checked_duration_since(*last_request_time) {
//             if time_passed < Duration::from_secs(1) {
//                 let sleep_duration = Duration::from_secs(1) - time_passed;
//                 std::thread::sleep(sleep_duration);
//             }
//         }

//         // Update the last request time
//         *last_request_time = Instant::now();
//     }
// }

//     // https://musicbrainz.org/doc/MusicBrainz_API (Lookups)
//     // > Note that the number of linked entities returned is always limited to 25. 
//     // > If you need the remaining results, you will have to perform a browse request. 
//     fn get(&self, 
//         model: &Model, 
//         network_mode: NetworkMode,
//         ctx: &PluginContext,
//     ) -> Result<Option<Model>> {
//         if network_mode != NetworkMode::Online {
//             return Err(Error::msg("Offline."))
//         }

//         match model {
//             Model::Artist(artist) => {
//                 let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;                
//                 let url = format!("https://musicbrainz.org/ws/2/artist/{}?fmt=json&inc=aliases+annotation+genres+ratings+tags+url-rels", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }
//                 let artist = response.json::<musicbrainz_rs::entity::artist::Artist>()?;
//                 let artist = Artist::from(ArtistConverter::from(artist.clone()));
//                 Ok(Some(artist.model()))
//             },

//             Model::ReleaseGroup(r) => {
//                 let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
//                 let url = format!("https://musicbrainz.org/ws/2/release-group/{}?fmt=json&inc=aliases+annotation+artists+genres+releases+ratings+tags+url-rels", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }        
//                 let release_group = response.json::<musicbrainz_rs::entity::release_group::ReleaseGroup>()?;
//                 let release_group = ReleaseGroup::from(ReleaseGroupConverter::from(release_group.clone()));
//                 Ok(Some(release_group.model()))
//             },

//             Model::Release(r) => {
//                 let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
//                 let url = format!("https://musicbrainz.org/ws/2/release/{}?fmt=json&inc=aliases+annotation+artists+genres+media+ratings+recordings+release-groups+tags+url-rels", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }        
//                 let release = response.json::<musicbrainz_rs::entity::release::Release>()?;
//                 let release = Release::from(ReleaseConverter::from(release.clone()));
//                 Ok(Some(release.model()))
//             },

//             Model::Recording(r) => {
//                 let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
//                 let url = format!("https://musicbrainz.org/ws/2/recording/{}?fmt=json&inc=aliases+annotation+artists+genres+isrcs+ratings+releases+release-groups+tags+url-rels", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }        
//                 let recording = response.json::<musicbrainz_rs::entity::recording::Recording>()?;
//                 let recording = Recording::from(RecordingConverter::from(recording.clone()));
//                 Ok(Some(recording.model()))
//             },
            
//             _ => Ok(None),
//         }
//     }

//     // https://musicbrainz.org/doc/MusicBrainz_API (Browse)
//     fn list(
//         &self,
//         list_of: &Model,
//         related_to: &Option<Model>,
//         network_mode: NetworkMode,
//         ctx: &PluginContext,
//     ) -> Result<Box<dyn Iterator<Item = Model>>> {
//         // TODO handle paging
//         if network_mode != NetworkMode::Online {
//             return Err(Error::msg("Offline."))
//         }
//         match (list_of, related_to) {
//             (Model::ReleaseGroup(_), Some(Model::Artist(artist))) => {                
//                 let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
//                 let url = format!("https://musicbrainz.org/ws/2/release-group?fmt=json&offset=0&limit=100&artist={}&inc=artist-credits", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }        
//                 let release_groups = response.json::<ReleaseGroups>()?;
//                 let iter = release_groups.release_groups.into_iter()
//                     .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                     .map(|src| src.model());
//                 Ok(Box::new(iter))
//             },
//             (Model::Release(_), Some(Model::Artist(artist))) => {                
//                 let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
//                 let url = format!("https://musicbrainz.org/ws/2/release?fmt=json&offset=0&limit=100&artist={}&inc=artist-credits+labels+recordings+release-groups+media+discids+isrcs", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }
//                 let releases = response.json::<Releases>()?;
//                 let iter = releases.releases.into_iter()
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(|src| src.model());
//                 Ok(Box::new(iter))
//             },
//             (Model::Release(_), Some(Model::ReleaseGroup(release_group))) => {                
//                 let mbid = release_group.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;
//                 let url = format!("https://musicbrainz.org/ws/2/release?fmt=json&offset=0&limit=100&release-group={}&inc=artist-credits+labels+recordings+release-groups+media+discids+isrcs", mbid);
//                 let response = ctx.get(self, &url)?;
//                 if !response.cached() {
//                     self.enforce_rate_limit();
//                 }
//                 let releases = response.json::<Releases>()?;
//                 let iter = releases.releases.into_iter()
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(|src| src.model());
//                 Ok(Box::new(iter))
//             },
//             _ => {
//                 log::debug!("list({}, {}) not implemented", 
//                     list_of.entity().type_name(), 
//                     related_to.clone().map(|r| r.entity().type_name()).unwrap_or_default());
//                 Err(Error::msg("Not implemented."))
//             },
//         }        
//     }

//     // TODO I want to return scores, or at least filter by them.
//     fn search(&self, 
//         query: &str, 
//         network_mode: NetworkMode,
//         ctx: &PluginContext,
//     ) -> Result<Box<dyn Iterator<Item = Model>>> {
//         if network_mode != NetworkMode::Online {
//             return Err(Error::msg("Offline."))
//         }
    
//         let iter = std::iter::empty();

//         let url = format!("https://musicbrainz.org/ws/2/artist/?query={}&fmt=json", &query);
//         let response = ctx.get(self, &url).unwrap();
//         if !response.cached() {
//             self.enforce_rate_limit();
//         }
//         let results = response.json::<ArtistResults>().unwrap();
//         let models = results.artists.into_iter()
//             .map(|src| Artist::from(ArtistConverter::from(src.clone())))
//             .map(|src| src.model());
//         let iter = iter.chain(models);

//         let url = format!("https://musicbrainz.org/ws/2/release-group/?query={}&fmt=json", &query);
//         if !response.cached() {
//             self.enforce_rate_limit();
//         }
//         let response = ctx.get(self, &url)?;
//         let results = response.json::<ReleaseGroupResults>()?;
//         let models = results.release_groups.into_iter()
//             .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//             .map(|src| src.model());
//         let iter = iter.chain(models);

//         let url = format!("https://musicbrainz.org/ws/2/recording/?query={}&fmt=json", &query);
//         if !response.cached() {
//             self.enforce_rate_limit();
//         }
//         let response = ctx.get(self, &url)?;
//         let results = response.json::<RecordingResults>()?;
//         let models = results.recordings.into_iter()
//             .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//             .map(|src| src.model());
//         let iter = iter.chain(models);

//         Ok(Box::new(iter))
//     }    
// }

// #[cfg(test)]
// mod tests {
//     use dimple_core::model::{Artist, Entity, KnownIds, Medium, Model, Recording, Release, ReleaseGroup, Track};
//     use dimple_librarian::plugin::{NetworkMode, Plugin, PluginContext};
//     use musicbrainz_rs::entity::release_group;

//     use crate::MusicBrainzPlugin;

//     #[test]
//     fn get_artist() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let artist = Artist {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let artist = plugin.get(&artist.model(), NetworkMode::Online, &ctx).unwrap().unwrap();
//     }

//     #[test]
//     fn get_release_group() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let release_group = ReleaseGroup {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("a96550cd-c202-326d-9593-313f72399ad5".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let release_group = plugin.get(&release_group.model(), NetworkMode::Online, &ctx).unwrap().unwrap();
//     }

//     #[test]
//     fn get_release() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let release = Release {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("006cb56d-6eff-4b7d-853f-ecd2db97f3b2".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let release = plugin.get(&release.model(), NetworkMode::Online, &ctx).unwrap().unwrap();
//     }

//     #[test]
//     fn get_recording() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let recording = Recording {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("70ac6733-068f-4613-b06b-bea17cfbcc30".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let recording = plugin.get(&recording.model(), NetworkMode::Online, &ctx).unwrap().unwrap();
//     }

//     #[test]
//     fn list_artist_releases() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let artist = Artist {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let releases: Vec<Release> = plugin.list(&Release::default().model(), 
//             &Some(artist.model()), NetworkMode::Online, &ctx)
//             .unwrap()
//             .map(|model| Release::from(model))
//             .collect();
//     }

//     #[test]
//     fn list_release_group_releases() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let release_group = ReleaseGroup {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("f44f4f73-a714-31a1-a4b8-bfcaaf311f50".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let releases: Vec<Release> = plugin.list(&Release::default().model(), 
//             &Some(release_group.model()), NetworkMode::Online, &ctx)
//             .unwrap()
//             .map(|model| Release::from(model))
//             .collect();
//     }

//     #[test]
//     fn search() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let results: Vec<Model> = plugin.search("Nirvana", NetworkMode::Online, &ctx).unwrap().collect();
//     }

//     #[test]
//     fn tree() {
//         let plugin = MusicBrainzPlugin::default();
//         let ctx = PluginContext::default();
//         let artist = Artist {
//             known_ids: KnownIds {
//                 musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
//                 ..Default::default()
//             },
//             ..Default::default()
//         };
//         let artist: Artist = plugin.get(&artist.model(), NetworkMode::Online, &ctx).unwrap().unwrap().into();
//         println!("{} {:?}", 
//             artist.name.clone().unwrap_or_default(),
//             artist.links,
//         );
//         let release_groups = plugin.list(&ReleaseGroup::default().model(), &Some(artist.model()), NetworkMode::Online, &ctx).unwrap();
//         for release_group in release_groups {
//             let release_group: ReleaseGroup = release_group.into();
//             println!("  {} {:?}", 
//                 release_group.title.clone().unwrap_or_default(),
//                 release_group.links,
//             );
//             let releases = plugin.list(&Release::default().model(), &Some(release_group.model()), NetworkMode::Online, &ctx).unwrap();
//             for release in releases {
//                 let release: Release = release.into();
//                 println!("    {} [{}] {} {:?}", 
//                     release.title.clone().unwrap_or_default(),
//                     release.country.clone().unwrap_or_default(),
//                     release.date.clone().unwrap_or_default(),
//                     release.links,
//                 );
//                 // let media = plugin.list(&Medium::default().model(), &Some(release.model()), NetworkMode::Online).unwrap();
//                 for medium in release.media {
//                     let medium: Medium = medium.into();
//                     println!("      {} / {} {}", 
//                         medium.position.unwrap_or_default(),
//                         medium.disc_count.unwrap_or_default(),
//                         medium.format.clone().unwrap_or_default(),
//                     );
//                     // let tracks = plugin.list(&Track::default().model(), &Some(medium.model()), NetworkMode::Online).unwrap();
//                     for track in medium.tracks {
//                         let track: Track = track.into();
//                         println!("        {} {}", 
//                             track.position.unwrap_or_default(),
//                             track.title.unwrap_or_default(),
//                         );

//                         let recording = track.recording;
//                         println!("            {} {} {:?}", 
//                             recording.title.unwrap_or_default(),
//                             recording.length.unwrap_or_default() / 1000,
//                             recording.links
//                         );
//                     }
//                 }
//             }
//         }
//     }    
// }
















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
