use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use anyhow::Result;
use musicbrainz_rs::entity::release_group;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{librarian::{ArtistMetadata, ReleaseGroupMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Release, ReleaseGroup, Track}, plugins::mb_converters::{ReleaseConverter, ReleaseGroupConverter}};

use super::{mb_converters::ArtistConverter, plugin::Plugin, plugins::Plugins};

// https://musicbrainz.org/doc/MusicBrainz_API
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

    fn search(&self, host: &Plugins, library: &Library, query: &str) 
        -> Result<crate::librarian::SearchResults, anyhow::Error> {
        
        // http://musicbrainz.org/ws/2/artist/?query=artist:klok
        let url = format!("https://musicbrainz.org/ws/2/artist/?fmt=json&query={query}");
        let mb_results: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::artist::Artist> = self.get(host, &url)?;
        let artists: Vec<ArtistMetadata> = mb_results.entities.into_iter().map(|e| ArtistConverter::from(e).into()).collect();

        // http://musicbrainz.org/ws/2/release-group/?fmt=json&query=master+of+puppets
        let url = format!("https://musicbrainz.org/ws/2/release-group/?fmt=json&query={query}");
        let mb_results: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release_group::ReleaseGroup> = self.get(host, &url)?;
        let release_groups: Vec<ReleaseGroupMetadata> = mb_results.entities.into_iter().map(|e| ReleaseGroupConverter::from(e).into()).collect(); 

        // let url = format!("https://musicbrainz.org/ws/2/release/?fmt=json&query={query}");
        // let mb_results: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release> = self.get(host, &url)?;
        // let releases: Vec<ReleaseMetadata> = mb_results.entities.into_iter().map(|e| ReleaseConverter::from(e).into()).collect(); 

        // TODO no genres search, just ship the list

        // TODO recordings
        // let url = format!("https://musicbrainz.org/ws/2/recording/?fmt=json&query={query}");
        // let mb_results: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::recording::Recording> = self.get(host, &url)?;
        // let releases: Vec<ReleaseMetadata> = mb_results.entities.into_iter().map(|e| ReleaseConverter::from(e).into()).collect();

        Ok(SearchResults {
            artists,
            release_groups,
            ..Default::default()
        })
    }

    fn artist_metadata(&self, plugins: &Plugins, _library: &Library, artist: &Artist) 
        -> Result<Option<ArtistMetadata>, anyhow::Error> {

        if let Some(mbid) = artist.musicbrainz_id.clone() {
            let url = format!("https://musicbrainz.org/ws/2/artist/{mbid}?fmt=json&inc=genres+url-rels");
            let mb_artist: musicbrainz_rs::entity::artist::Artist = self.get(plugins, &url)?;
            let artist_metadata: ArtistMetadata = ArtistConverter::from(mb_artist).into();
            return Ok(Some(artist_metadata))
        }
        Ok(None)
    }

    fn artist_release_groups(&self, plugins: &Plugins, library: &Library, artist: &Artist) 
        -> Result<Vec<ReleaseGroupMetadata>> {
        
        if let Some(mbid) = artist.musicbrainz_id.clone() {
            let limit: usize = 100;
            let mut offset: usize = 0;
            let mut results: Vec<ReleaseGroupMetadata> = vec![];
            loop {
                let url = format!(concat!(
                    "https://musicbrainz.org/ws/2/release-group",
                    "?fmt=json",
                    "&artist={}",
                    "&inc=artist-credits+genres+url-rels+ratings",
                    "&offset={}",
                    "&limit={}"), mbid, offset, limit);
                let response: ReleaseGroupsResponse = self.get(plugins, &url)?;
                if response.release_groups.is_empty() {
                    break
                }
                else {
                    offset += response.release_groups.len();
                }
                response.release_groups.into_iter()
                    .map(|src| ReleaseGroupMetadata::from(ReleaseGroupConverter::from(src.clone())))
                    .for_each(|r| results.push(r));
            }
            return Ok(results)
        }
        Ok(vec![])
    }

    fn artist_releases(&self, plugins: &Plugins, library: &Library, artist: &Artist) 
        -> Result<Vec<ReleaseMetadata>> {
        
        if let Some(mbid) = artist.musicbrainz_id.clone() {
            let limit: usize = 100;
            let mut offset: usize = 0;
            let mut results: Vec<ReleaseMetadata> = vec![];
            loop {
                let url = format!(concat!(
                    "https://musicbrainz.org/ws/2/release",
                    "?fmt=json",
                    "&artist={}",
                    "&inc=artist-credits+labels+recordings+release-groups+media+discids+isrcs+genres+url-rels+ratings",
                    "&offset={}",
                    "&limit={}"), mbid, offset, limit);
                let response: ReleasesResponse = self.get(plugins, &url)?;
                if response.releases.is_empty() {
                    break
                }
                else {
                    offset += response.releases.len();
                }
                response.releases.into_iter()
                    .map(|src| ReleaseMetadata::from(ReleaseConverter::from(src.clone())))
                    .for_each(|r| results.push(r));
            }
            return Ok(results)
        }
        Ok(vec![])
    }

    fn release_group_metadata(&self, host: &Plugins, _library: &Library, release_group: &ReleaseGroup) 
        -> Result<Option<ReleaseGroupMetadata>, anyhow::Error> {

        if let Some(mbid) = release_group.musicbrainz_id.clone() {
            let url = format!(concat!(
                "https://musicbrainz.org/ws/2/release-group/{}",
                "?fmt=json",
                "&inc=artist-credits+releases+media+discids+genres+url-rels+ratings",
            ), mbid);
            let mb_group: musicbrainz_rs::entity::release_group::ReleaseGroup = self.get(host, &url)?;
            let release_group_metadata: ReleaseGroupMetadata = ReleaseGroupConverter::from(mb_group).into();
            return Ok(Some(release_group_metadata))
        }
        Ok(None)
    }

    fn release_group_releases(&self, plugins: &Plugins, library: &Library, release_group: &ReleaseGroup) -> Result<Vec<ReleaseMetadata>> {
        if let Some(mbid) = release_group.musicbrainz_id.clone() {
            let limit: usize = 100;
            let mut offset: usize = 0;
            let mut releases: Vec<ReleaseMetadata> = vec![];
            loop {
                let url = format!(concat!(
                    "https://musicbrainz.org/ws/2/release",
                    "?fmt=json",
                    "&release-group={}",
                    "&status=official",
                    "&inc=artist-credits+recordings+media+discids+isrcs+genres+url-rels+ratings",
                    "&offset={}",
                    "&limit={}"), mbid, offset, limit);
                let releases_response: ReleasesResponse = self.get(plugins, &url)?;
                if releases_response.releases.is_empty() {
                    break
                }
                else {
                    offset += releases_response.releases.len();
                }
                releases_response.releases.into_iter()
                    .map(|src| ReleaseMetadata::from(ReleaseConverter::from(src.clone())))
                    .for_each(|r| releases.push(r));
            }
            Ok(releases)
        }
        else {
            Ok(vec![])
        }
    }        

    fn release_metadata(&self, host: &Plugins, _library: &Library, release: &Release) 
        -> Result<Option<ReleaseMetadata>, anyhow::Error> {

        if let Some(mbid) = release.musicbrainz_id.clone() {
            let url = format!(concat!(
                "https://musicbrainz.org/ws/2/release/{}",
                "?fmt=json",
                "&inc=artist-credits+recordings+release-groups+media+discids+isrcs+genres+url-rels+ratings",
            ), mbid);
            let mb_release: musicbrainz_rs::entity::release::Release = self.get(host, &url)?;
            let release_metadata: ReleaseMetadata = ReleaseConverter::from(mb_release).into();
            return Ok(Some(release_metadata))
        }
        Ok(None)
    }

    fn track_metadata(&self, host: &Plugins, _library: &Library, track: &Track) 
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
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ReleasesResponse {
    pub releases: Vec<musicbrainz_rs::entity::release::Release>,
}
    
    
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseGroupsResponse {
    pub release_groups: Vec<musicbrainz_rs::entity::release_group::ReleaseGroup>,
}
    
impl MusicBrainzPlugin {
    /// https://community.metabrainz.org/t/api-rate-limit/725714/3
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

    fn get<T: DeserializeOwned>(&self, host: &Plugins, url: &str) -> Result<T, anyhow::Error> {
        // TODO handle rate limit overage, with backoff
        let response = host.get(url)?;
        if !response.cached() {
            self.enforce_rate_limit();
        }
        response.json()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct MusicBrainzPluginConfig {    
    pub url: String,
    pub username: String,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use crate::{librarian::{self, ArtistMetadata}, library::Library, model::{Artist, ReleaseGroup}, plugins::{musicbrainz::MusicBrainzPlugin, plugin::Plugin as _, plugins::Plugins}};

    #[test]
    fn test_search() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let results = plugin.search(&plugins, &library, "death clock").unwrap();
        assert_eq!(results.artists[0].artist.name, Some("Dethklok".to_string()));
    }    

    #[test]
    fn test_artist_metadata() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let artist_metadata = plugin.artist_metadata(&plugins, &library, &Artist {
            musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
            ..Default::default()
        }).unwrap().unwrap();
        assert_eq!(artist_metadata.artist.name, Some("We Were Heading North".to_string()));
        assert!(artist_metadata.links.len() >= 2);
        assert!(artist_metadata.genres.len() >= 1);
    }

    #[test]
    fn test_artist_release_groups() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let artist = Artist {
            musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
            ..Default::default()
        };
        let release_groups = plugin.artist_release_groups(&plugins, &library, &artist).unwrap();
        assert!(release_groups.len() >= 3);
        assert_eq!(release_groups[0].release_group.title, Some("Lightness".to_string()));
    }

    #[test]
    fn test_artist_releases() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let artist = Artist {
            musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
            ..Default::default()
        };
        let releases = plugin.artist_releases(&plugins, &library, &artist).unwrap();
        assert!(releases.len() >= 3);
        assert_eq!(releases[0].release.title, Some("three".to_string()));
    }

    #[test]
    fn test_release_group_metadata() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let release_group = plugin.release_group_metadata(&plugins, &library, &ReleaseGroup {
            musicbrainz_id: Some("bded9aa5-c420-35bf-912c-94bd25283d0f".to_string()),
            ..Default::default()
        }).unwrap().unwrap();
        assert_eq!(release_group.release_group.title, Some("The Youth Are Getting Restless".to_string()));
    }

    #[test]
    fn test_release_group_releases() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();

        let releases = plugin.release_group_releases(&plugins, &library, &ReleaseGroup {
            musicbrainz_id: Some("bded9aa5-c420-35bf-912c-94bd25283d0f".to_string()),
            ..Default::default()
        }).unwrap();
        assert_eq!(releases[0].release.date, Some("1990-01-01".to_string()));
    }
}
