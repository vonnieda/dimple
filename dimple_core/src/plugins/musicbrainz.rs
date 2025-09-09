use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use anyhow::Result;
use musicbrainz_rs::entity::release_group;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{librarian::{ArtistMetadata, ReleaseGroupMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Release, ReleaseGroup, Track}, plugins::converters::{ReleaseConverter, ReleaseGroupConverter}};

use super::{converters::ArtistConverter, plugin::Plugin, plugins::Plugins};

// https://musicbrainz.org/doc/MusicBrainz_API
// Subqueries
// The inc= parameter allows you to request more information to be included about the entity. Any of the entities directly linked to the entity can be included.
//  /ws/2/area
//  /ws/2/artist            recordings, releases, release-groups, works
//  /ws/2/collection        user-collections (includes private collections, requires authentication)
//  /ws/2/event
//  /ws/2/genre
//  /ws/2/instrument
//  /ws/2/label             releases
//  /ws/2/place
//  /ws/2/recording         artists, releases, release-groups, isrcs, url-rels
//  /ws/2/release           artists, collections, labels, recordings, release-groups
//  /ws/2/release-group     artists, releases
//  /ws/2/series
//  /ws/2/work
//  /ws/2/url

// Some additional inc= parameters are supported to specify how much of the data about the linked entities should be included:
//  - discids           include discids for all media in the releases
//  - media             include media for all releases, this includes the # of tracks on each medium and its format.
//  - isrcs             include isrcs for all recordings
//  - artist-credits    include artists credits for all releases and recordings
//  - various-artists   include only those releases where the artist appears on one of the tracks, 
//                      but not in the artist credit for the release itself (this is only valid on a
//                      /ws/2/artist?inc=releases request).

// Misc inc= arguments
// - aliases                   include artist, label, area or work aliases; treat these as a set, as they are not deliberately ordered
// - annotation                include annotation
// - tags, ratings             include tags and/or ratings for the entity
// - user-tags, user-ratings   same as above, but only return the tags and/or ratings submitted by the specified user
// - genres, user-genres       include genres (tags in the genres list): either all or the ones submitted by the user, respectively
// 

// The following list shows which linked entities you can use in a browse request:
//  /ws/2/area              collection
//  /ws/2/artist            area, collection, recording, release, release-group, work
//  /ws/2/collection        area, artist, editor, event, label, place, recording, release, release-group, work
//  /ws/2/event             area, artist, collection, place
//  /ws/2/genre             collection
//  /ws/2/instrument        collection
//  /ws/2/label             area, collection, release
//  /ws/2/place             area, collection
//  /ws/2/recording         artist, collection, release, work
//  /ws/2/release           area, artist, collection, label, track, track_artist, recording, release-group
//  /ws/2/release-group     artist, collection, release
//  /ws/2/series            collection
//  /ws/2/work              artist, collection

// Just like with normal lookup requests, the server can be instructed to include more data about the entity using an 'inc=' argument. Supported values for inc= are:
//  /ws/2/area              aliases
//  /ws/2/artist            aliases
//  /ws/2/event             aliases
//  /ws/2/instrument        aliases
//  /ws/2/label             aliases
//  /ws/2/place             aliases
//  /ws/2/recording         artist-credits, isrcs
//  /ws/2/release           artist-credits, labels, recordings, release-groups, media, discids, isrcs (with recordings)
//  /ws/2/release-group     artist-credits
//  /ws/2/series            aliases
//  /ws/2/work              aliases
//  /ws/2/area              aliases
//  /ws/2/url               (only relationship includes)

// In addition to the inc= values listed above, all entities support:
//  annotation, tags, user-tags, genres, user-genres

// All entities except area, place, release, and series support:
//  ratings, user-ratings

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

    fn artist_metadata(&self, plugins: &Plugins, _library: &Library, artist: &Artist) 
        -> Result<Option<ArtistMetadata>, anyhow::Error> {

        if let Some(mbid) = artist.musicbrainz_id.clone() {
            let url = format!("https://musicbrainz.org/ws/2/artist/{mbid}?fmt=json&inc=genres+url-rels");
            let mb_artist: musicbrainz_rs::entity::artist::Artist = self.get(plugins, &url)?;
            let mut artist_metadata: ArtistMetadata = ArtistConverter::from(mb_artist).into();
            // artist_metadata.releases = self.artist_releases(plugins, &mbid)?;
            return Ok(Some(artist_metadata))
        }
        Ok(None)
    }

    fn release_metadata(&self, host: &Plugins, _library: &Library, release: &Release) 
        -> Result<Option<ReleaseMetadata>, anyhow::Error> {

        if let Some(mbid) = release.musicbrainz_id.clone() {
            // Browse
            //  /ws/2/release           area, artist, collection, label, track, track_artist, recording, release-group
            // Inc
            //  /ws/2/release           artist-credits, labels, recordings, release-groups, media, discids, isrcs (with recordings)
            // In addition to the inc= values listed above, all entities support:
            //  annotation, tags, user-tags, genres, user-genres
            // All entities except area, place, release, and series support:
            //  ratings, user-ratings
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

    // fn release_group_metadata(&self, host: &Plugins, _library: &Library, release_group: &ReleaseGroup) 
    //     -> Result<Option<ReleaseGroupMetadata>, anyhow::Error> {

    //     if let Some(mbid) = release_group.musicbrainz_id.clone() {
    //         // https://musicbrainz.org/doc/MusicBrainz_API
    //         // Subqueries
    //         // The inc= parameter allows you to request more information to be included about the entity. Any of the entities directly linked to the entity can be included.
    //         //  /ws/2/release-group     artists, releases

    //         // Some additional inc= parameters are supported to specify how much of the data about the linked entities should be included:
    //         //  - discids           include discids for all media in the releases
    //         //  - media             include media for all releases, this includes the # of tracks on each medium and its format.
    //         //  - isrcs             include isrcs for all recordings
    //         //  - artist-credits    include artists credits for all releases and recordings

    //         // Misc inc= arguments
    //         // - aliases                   include artist, label, area or work aliases; treat these as a set, as they are not deliberately ordered
    //         // - annotation                include annotation
    //         // - tags, ratings             include tags and/or ratings for the entity
    //         // - user-tags, user-ratings   same as above, but only return the tags and/or ratings submitted by the specified user
    //         // - genres, user-genres       include genres (tags in the genres list): either all or the ones submitted by the user, respectively
    //         // 

    //         // The following list shows which linked entities you can use in a browse request:
    //         //  /ws/2/release-group     artist, collection, release

    //         // Just like with normal lookup requests, the server can be instructed to include more data about the entity using an 'inc=' argument. Supported values for inc= are:
    //         //  /ws/2/release-group     artist-credits

    //         // In addition to the inc= values listed above, all entities support:
    //         //  annotation, tags, user-tags, genres, user-genres

    //         // All entities except area, place, release, and series support:
    //         //  ratings, user-ratings
    //         let url = format!(concat!(
    //             "https://musicbrainz.org/ws/2/release-group/{}",
    //             "?fmt=json",
    //             "&inc=artist-credits+recordings+release-groups+media+discids+isrcs+genres+url-rels+ratings",
    //         ), mbid);
    //         let mb_release: musicbrainz_rs::entity::release::Release = self.get(host, &url)?;
    //         let release_metadata: ReleaseMetadata = ReleaseConverter::from(mb_release).into();
    //         return Ok(Some(release_metadata))
    //     }
    //     Ok(None)
    // }

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
    // Release (Group) Type and Status
    // Any query which includes release groups in the results can be filtered
    // to only include release groups of a certain type. Any query which
    // includes releases in the results can be filtered to only include 
    // releases of a certain type and/or status. Valid values are:
    //  status     official, promotion, bootleg, pseudo-release, withdrawn, cancelled.
    //  type       album, single, ep, broadcast, other (primary types) / audio drama, audiobook, compilation, demo, dj-mix, field recording, interview, live, mixtape/street, remix, soundtrack, spokenword (secondary types).
    // See the release status documentation and the release group type 
    // documentation for info on what these values mean.
    // Additionally, browsing release groups via artist supports a special 
    // filter to show the same release groups as in the default website 
    // overview (excluding ones that contain only releases of status 
    // promotional, bootleg or pseudo-release). Valid values are:
    //  release-group-status     website-default, all
    fn artist_releases(&self, plugins: &Plugins, mbid: &str) -> Result<Vec<ReleaseMetadata>> {
        let limit: usize = 100;
        let mut offset: usize = 0;
        let mut releases: Vec<ReleaseMetadata> = vec![];
        loop {
            let url = format!(concat!(
                "https://musicbrainz.org/ws/2/release",
                "?fmt=json",
                "&artist={}",
                "&status=official",
                "&inc=artist-credits+recordings+release-groups+media+discids+isrcs+genres+url-rels+ratings",
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

    fn artist_release_groups(&self, plugins: &Plugins, mbid: &str) -> Result<Vec<ReleaseGroupMetadata>> {
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
        Ok(results)
    }

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
    use crate::{librarian::{self, ArtistMetadata}, library::Library, model::Artist, plugins::{musicbrainz::MusicBrainzPlugin, plugin::Plugin as _, plugins::Plugins}};

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
        // assert!(artist_metadata.releases.len() >= 3);
        // assert!(artist_metadata.releases[0].artists.len() >= 1);
        // assert!(artist_metadata.releases[0].release.release_group_musicbrainz_id.is_some());
        let _ = library.db.transaction(move |t| {
            librarian::merge_artist_metadata(t, &artist_metadata, None)
        });
    }

        #[test]
    fn test_artist_release_groups() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        // Metallica 65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab
        // We Were Heading North 73084492-3e59-4b7f-aa65-572a9d7691d5
        let release_groups = plugin.artist_release_groups(&plugins, "65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab").unwrap();
        // let artist_metadata = plugin.artist_metadata(&plugins, &library, &Artist {
        //     musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
        //     ..Default::default()
        // }).unwrap().unwrap();
        // assert_eq!(artist_metadata.artist.name, Some("We Were Heading North".to_string()));
        // assert!(artist_metadata.links.len() >= 2);
        // assert!(artist_metadata.genres.len() >= 1);
        // assert!(artist_metadata.releases.len() >= 3);
        // assert!(artist_metadata.releases[0].artists.len() >= 1);
        // assert!(artist_metadata.releases[0].release.release_group_musicbrainz_id.is_some());
        // let _ = library.db.transaction(move |t| {
        //     librarian::merge_artist_metadata(t, &artist_metadata, None)
        // });
    }

    #[test]
    fn test_search() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let results = plugin.search(&plugins, &library, "death clock").unwrap();
        // dbg!(results);
    }    
}
