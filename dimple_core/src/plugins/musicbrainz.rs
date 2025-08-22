use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Release, Track}, plugins::converters::ReleaseConverter};

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

    fn artist_metadata(&self, host: &Plugins, _library: &Library, artist: &Artist) 
        -> Result<Option<ArtistMetadata>, anyhow::Error> {

        if let Some(mbid) = artist.musicbrainz_id.clone() {
            let url = format!("https://musicbrainz.org/ws/2/artist/{mbid}?fmt=json&inc=releases+release-groups+artist-credits+genres+url-rels");
            let mb_artist: musicbrainz_rs::entity::artist::Artist = self.get(host, &url)?;
            let artist_metadata: ArtistMetadata = ArtistConverter::from(mb_artist).into();
            return Ok(Some(artist_metadata))
        }
        Ok(None)
    }

    fn release_metadata(&self, host: &Plugins, _library: &Library, release: &Release) 
        -> Result<Option<ReleaseMetadata>, anyhow::Error> {

        if let Some(mbid) = release.musicbrainz_id.clone() {
            // TODO artists? artist-credits?
            let url = format!("https://musicbrainz.org/ws/2/release/{mbid}?fmt=json&inc=aliases+annotation+artists+genres+media+ratings+recordings+release-groups+tags+url-rels");
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
    
    fn search(&self, host: &Plugins, library: &Library, query: &str) 
        -> Result<crate::librarian::SearchResults, anyhow::Error> {
        
        // http://musicbrainz.org/ws/2/artist/?query=artist:klok
        let url = format!("https://musicbrainz.org/ws/2/artist/?fmt=json&query={query}");
        let mb_results: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::artist::Artist> = self.get(host, &url)?;
        let artists: Vec<ArtistMetadata> = mb_results.entities.into_iter().map(|e| ArtistConverter::from(e).into()).collect();

        let url = format!("https://musicbrainz.org/ws/2/release/?fmt=json&query={query}");
        let mb_results: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release> = self.get(host, &url)?;
        let releases: Vec<ReleaseMetadata> = mb_results.entities.into_iter().map(|e| ReleaseConverter::from(e).into()).collect();

        Ok(SearchResults {
            artists: artists.into_iter().map(|e| e.artist).collect(),
            releases: releases.into_iter().map(|e| e.release).collect(),
            ..Default::default()
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ReleasesResponse {
    pub releases: Vec<musicbrainz_rs::entity::release::Release>,
}
    
impl MusicBrainzPlugin {
    fn artist_releases(&self, plugins: &Plugins, mbid: &str) -> Result<Vec<ReleaseMetadata>> {
        let limit: usize = 25;
        let mut offset: usize = 0;
        let mut releases: Vec<ReleaseMetadata> = vec![];
        loop {
            let url = format!("https://musicbrainz.org/ws/2/release?artist={mbid}&status=official&inc=artist-credits+labels+recordings+release-groups+media+discids+isrcs+tags+genres+url-rels&fmt=json&offset={offset}&limit={limit}");
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
    fn it_works() {
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
        assert!(artist_metadata.releases.len() >= 3);
        assert!(artist_metadata.releases[0].artists.len() >= 1);
        dump_artist_metadata(&artist_metadata);
        let _ = library.db.transaction(move |t| {
            librarian::merge_artist_metadata(t, &artist_metadata, None)
        });
        dump_library(&library);
    }

    fn dump_artist_metadata(artist_metadata: &ArtistMetadata) {
        println!("{} {}", artist_metadata.artist.name.as_deref().unwrap_or_default(), artist_metadata.artist.musicbrainz_id.as_deref().unwrap_or_default());
        println!("Releases:");
        for release in &artist_metadata.releases {
            println!("  {} {}", &release.release.title.as_deref().unwrap_or_default(), release.release.musicbrainz_id.as_deref().unwrap_or_default());
            println!("  Artists:");
            for artist in &release.artists {
                println!("    {} {}", &artist.artist.name.as_deref().unwrap_or_default(), &artist.artist.musicbrainz_id.as_deref().unwrap_or_default());
            }
            println!("  Tracks:");
            for track in &release.tracks {
                println!("    {} {}", &track.track.position.unwrap_or_default(), &track.track.title.as_deref().unwrap_or_default());
                println!("    Artists:");
                for artist in &track.artists {
                    println!("      {} {}", &artist.artist.name.as_deref().unwrap_or_default(), &artist.artist.musicbrainz_id.as_deref().unwrap_or_default());
                }
                println!("    Genres:");
                for genre in &track.genres {
                    println!("      {} {}", &genre.name.as_deref().unwrap_or_default(), &genre.musicbrainz_id.as_deref().unwrap_or_default());
                }
            }
        }
    }
    
    fn dump_library(library: &Library) {
        for artist in library.list::<Artist>() {
            println!("{}", artist.name.as_deref().unwrap_or_default());
            for release in artist.releases(library) {
                println!("  {}", &release.title.as_deref().unwrap_or_default());
                for track in release.tracks(library) {
                    println!("    {} {}", &track.position.unwrap_or_default(), &track.title.as_deref().unwrap_or_default());
                }
            }
        }
    }

    #[test]
    fn search() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = MusicBrainzPlugin::default();
        let results = plugin.search(&plugins, &library, "death clock").unwrap();
        dbg!(results);
    }    
}
