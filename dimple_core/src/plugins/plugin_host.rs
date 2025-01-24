use std::sync::{Arc, Mutex, RwLock};

use lru::LruCache;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use crate::{library::Library, merge::CrdtRules, model::Model};

use super::{plugin::Plugin, USER_AGENT};

#[derive(Clone)]
pub struct PluginHost {
    // If I made the Vec contain Arc I could speed this up by not
    // locking while iterating.
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
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
    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    }

    // TODO Right now never returns None, make it so.
    pub fn metadata<T: Model + Clone + CrdtRules + 'static>(&self, library: &Library, model: &T) -> Option<T> {
        let mut model = model.clone();
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(plugin_metadata)) = plugin.metadata(self, library, &model) {
                let metadata = plugin_metadata.as_any().downcast_ref::<T>().unwrap().clone();
                model = CrdtRules::merge(model.clone(), metadata.clone());
            }
        }
        Some(model)
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
            log::info!("CACHED  [{:?}] {:?} {}", cached.status, cached.response.len(), url);
            return Ok(cached)
        }
        let client = self.client()?;
        let response = client.get(url).send()?;
        log::info!("FETCHED [{:?}] {:?} {}", 
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
    use crate::{
        library::Library,
        model::{Artist, ArtistRef, Track}, plugins::{example::ExamplePlugin, lrclib::LrclibPlugin, musicbrainz::MusicBrainzPlugin, wikidata::WikidataPlugin},
    };

    use super::PluginHost;

    #[test]
    fn it_works() {
        let plugins = PluginHost::default();
        plugins.add_plugin(Box::new(ExamplePlugin::default()));
        plugins.add_plugin(Box::new(LrclibPlugin::default()));

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

        assert!(plugins.metadata(&library, &track).is_some());
    }

    #[test]
    fn artist_metadata() {
        let _ = env_logger::try_init();
        let plugins = PluginHost::default();
        plugins.add_plugin(Box::new(ExamplePlugin::default()));
        plugins.add_plugin(Box::new(LrclibPlugin::default()));
        plugins.add_plugin(Box::new(MusicBrainzPlugin::default()));
        plugins.add_plugin(Box::new(WikidataPlugin::default()));

        let library = Library::open_memory();
        let artist = library.save(&Artist {
            name: Some("Metallica".to_string()),
            musicbrainz_id: Some("65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab".to_string()),
            ..Default::default()
        });

        assert!(plugins.metadata(&library, &artist).is_some());
    }
}

