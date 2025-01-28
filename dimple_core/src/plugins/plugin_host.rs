use std::sync::{Arc, Mutex, RwLock};

use lru::LruCache;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, TrackMetadata}, library::Library, merge::CrdtRules, model::{Artist, Model, Release, Track}};

use super::{plugin::Plugin, USER_AGENT};

#[derive(Clone)]
pub struct PluginHost {
    plugins: Arc<RwLock<Vec<Arc<dyn Plugin>>>>,
    cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
}

/// Right away, I need:
/// - lrclib lyrics (already have, fit API)
/// - musicbrainz links (needed for summary)
/// - wikidata summary
/// - tadb artist artwork
/// - fanart artist artwork
/// - caa release artwork
impl Default for PluginHost {
    fn default() -> Self {
        PluginHost {
            plugins: Default::default(),
            cache: Arc::new(Mutex::new(LruCache::unbounded())),
        }
    }
}

impl PluginHost {
    pub fn add_plugin(&self, plugin: Arc<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    }

    // // TODO Right now never returns None, make it so.
    // pub fn metadata<T: Model + Clone + CrdtRules + 'static>(&self, library: &Library, model: &T) -> Option<T> {
    //     let mut model = model.clone();
    //     for plugin in self.plugins.read().unwrap().iter() {
    //         if let Ok(Some(plugin_metadata)) = plugin.metadata(self, library, &model) {
    //             let metadata = plugin_metadata.as_any().downcast_ref::<T>().unwrap().clone();
    //             model = CrdtRules::merge(model.clone(), metadata.clone());
    //         }
    //     }
    //     Some(model)
    // }

    pub fn artist_metadata(&self, library: &Library, artist: &Artist) -> Vec<ArtistMetadata> {
        let mut results = vec![];
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(metadata)) = plugin.artist_metadata(self, library, artist) {
                results.push(metadata);
            }
        }
        results
    }

    pub fn release_metadata(&self, library: &Library, release: &Release) -> Vec<ReleaseMetadata> {
        let mut results = vec![];
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(metadata)) = plugin.release_metadata(self, library, release) {
                results.push(metadata);
            }
        }
        results
    }

    pub fn track_metadata(&self, library: &Library, track: &Track) -> Vec<TrackMetadata> {
        let mut results = vec![];
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(metadata)) = plugin.track_metadata(self, library, track) {
                results.push(metadata);
            }
        }
        results
    }

    pub fn image<T: Model>(&self, _library: &Library, _model: &T) -> Option<T> {
        todo!()
        // Some(self.plugins
        //     .read()
        //     .unwrap()
        //     .par_iter()
        //     .filter_map(|plugin| plugin.metadata(library, track))
        //     .reduce(Track::default, Track::merge))
    }

    pub fn client(&self) -> Result<Client, anyhow::Error> {
        Ok(Client::builder()
            .user_agent(USER_AGENT)
            .build()?)
    }

    pub fn get(&self, url: &str) -> Result<CachedResponse, anyhow::Error> {
        if let Some(cached) = self.cache_get(url) {
            log::debug!("CACHED  [{:?}] {:?} {}", cached.status, cached.response.len(), url);
            return Ok(cached)
        }
        let client = self.client()?;
        let response = client.get(url).send()?;
        log::debug!("FETCHED [{:?}] {:?} {}", 
            response.status().as_u16(), 
            response.content_length().unwrap_or_default(),
            url);
        let status = response.status().as_u16();
        let bytes = response.bytes()?;
        let cached = CachedResponse::new(bytes.to_vec(), false, status);
        self.cache_put(url, &cached);
        Ok(cached)
    }

    pub fn cache_get(&self, url: &str) -> Option<CachedResponse> {
        self.cache.lock().unwrap().get(url).cloned()
    }

    pub fn cache_put(&self, url: &str, response: &CachedResponse) {
        self.cache.lock().unwrap().put(url.to_owned(), response.clone());
    }
}

pub fn nempty(s: &String) -> Option<String> {
    if s.is_empty() {
        None
    }
    else {
        Some(s.to_string())
    }
}

#[derive(Clone)]
pub struct CachedResponse {
    response: Vec<u8>,
    cached: bool,
    status: u16,
}

impl CachedResponse {
    pub fn new(response: Vec<u8>, cached: bool, status: u16) -> Self {
        Self {
            response,
            cached,
            status,
        }
    }

    pub fn cached(&self) -> bool {
        self.cached
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, anyhow::Error> {
        Ok(serde_json::from_slice(&self.response)?)
    }    

    pub fn bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        return Ok(self.response.clone())
    }
}

#[cfg(test)]
mod tests { 
    use std::sync::Arc;

    use crate::{
        library::Library,
        model::{Artist, ArtistRef, Track}, plugins::{example::ExamplePlugin, lrclib::LrclibPlugin, musicbrainz::MusicBrainzPlugin, wikidata::WikidataPlugin},
    };

    use super::PluginHost;

    #[test]
    fn it_works() {
        let plugins = PluginHost::default();
        plugins.add_plugin(Arc::new(ExamplePlugin::default()));
        plugins.add_plugin(Arc::new(LrclibPlugin::default()));

        let library = Library::open_memory();
        let artist = library.save(&Artist {
            name: Some("Metallica".to_string()),
            ..Default::default()
        });
        let track = library.save(&Track {
            title: Some("Master of Puppets".to_string()),
            ..Default::default()
        });
        ArtistRef::attach(&library, &artist, &track);

        // assert!(plugins.track_metadata(&library, &track).is_some());
    }

    #[test]
    fn artist_metadata() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let artist = library.save(&Artist {
            musicbrainz_id: Some("6821bf3f-5d5b-4b0f-8fa4-79d2ab2d9219".to_string()),
            ..Default::default()
        });
        let plugins = PluginHost::default();
        plugins.add_plugin(Arc::new(ExamplePlugin::default()));
        plugins.add_plugin(Arc::new(LrclibPlugin::default()));
        plugins.add_plugin(Arc::new(MusicBrainzPlugin::default()));
        plugins.add_plugin(Arc::new(WikidataPlugin::default()));
        let metadata = plugins.artist_metadata(&library, &artist);
        // assert!(metadata.artist.name == Some("Blonde Redhead".to_string()));
        dbg!(metadata);
    }
}

