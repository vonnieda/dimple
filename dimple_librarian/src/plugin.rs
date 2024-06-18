// #![forbid(unsafe_code, future_incompatible)]
// #![deny(
//     missing_docs,
//     missing_debug_implementations,
//     missing_copy_implementations,
//     nonstandard_style,
//     unused_qualifications,
//     unused_import_braces,
//     unused_extern_crates,
//     trivial_casts,
//     trivial_numeric_casts
// )]

use std::{io::Read, time::Instant};

use anyhow::Result;
use dimple_core::model::{Entity, Model};

use colored::Colorize;
use reqwest::{blocking::{Client, Response}};
use serde::de::DeserializeOwned;

pub const USER_AGENT: &str = "Dimple/0.0.1 +https://github.com/vonnieda/dimple +jason@vonnieda.org";

pub trait Plugin: Send + Sync {
    fn name(&self) -> String;

    /// Load the model using its key. Returns None if no key is set, or if the
    /// key doesn't exist in the database.
    fn get(&self, model: &Model, network_mode: NetworkMode) -> Result<Option<Model>> {
        Ok(None)
    }

    /// Get a list of models that are related to the specified model. If None is
    /// specified list all models of the specified type.
    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
        network_mode: NetworkMode,
    ) -> Result<Box<dyn Iterator<Item = Model>>> {
        Ok(Box::new(std::iter::empty()))
    }

    fn search(&self, query: &str, network_mode: NetworkMode) 
        -> Result<Box<dyn Iterator<Item = Model>>> {
        Ok(Box::new(std::iter::empty()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    Online,
    Offline,
    // This might be a great way to handle refresh / cache flush
    Force,
}

pub struct PluginSupport {
}

pub struct RequestToken {
    tag: String,
    start_time: Instant,
    url: String,
}

impl PluginSupport {
    pub fn start_request(plugin: &dyn Plugin, url: &str) -> RequestToken {
        RequestToken {
            tag: plugin.name(),
            start_time: Instant::now(),
            url: url.to_owned(),
        }
    }

    pub fn end_request(token: RequestToken, status_code: Option<u16>, length: Option<u64>) {
        log::info!("{} [{:?}] {}ms {:?} {}", 
            token.tag, 
            status_code, 
            token.start_time.elapsed().as_millis(), 
            length,
            token.url.yellow());
    }

    /// The most common use of HTTP is going to be a simple get, so this
    /// will be a shortcut for that. I think I'll also need a client()
    /// that returns a pre-configured client that the plugin can use for
    /// more complex tasks.
    pub fn get(plugin: &dyn Plugin, url: &str) -> Result<CacheResponse> {
        // TODO use dirs, or better yet, the librarian path
        let cache = "./dimple-librarian-plugin-cache";
        if let Some(cached) = cacache::read_sync(cache, url).ok() {
            log::info!("{} {} (Cached) {}", 
                plugin.name(), 
                cached.len(),
                url.blue());
            return Ok(CacheResponse::new(cached, true))
        }
        let client = Client::builder()
            .user_agent(super::plugin::USER_AGENT)
            .build()?;
        let request_token = Self::start_request(plugin, &url);
        let response = client.get(url).send()?;
        Self::end_request(request_token, 
            Some(response.status().as_u16()), 
            response.content_length());
        let bytes = response.bytes()?;
        cacache::write_sync(cache, url, &bytes)?;
        return Ok(CacheResponse::new(bytes.to_vec(), false))
    }
}

pub struct CacheResponse {
    response: Vec<u8>,
    cached: bool,
}

impl CacheResponse {
    pub fn new(response: Vec<u8>, cached: bool) -> Self {
        Self {
            response,
            cached,
        }
    }

    pub fn cached(&self) -> bool {
        self.cached
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.response)?)
    }    

    pub fn bytes(&self) -> Result<Vec<u8>> {
        return Ok(self.response.clone())
    }
}

