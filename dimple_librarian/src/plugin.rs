use std::time::Instant;

use anyhow::Result;
use dimple_core::model::{Entity, Model};

use colored::Colorize;
use reqwest::blocking::{Client, Response};

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
            token.tag.yellow(), 
            status_code, 
            token.start_time.elapsed().as_millis(), 
            length,
            token.url);
    }

    /// The most common use of HTTP is going to be a simple get, so this
    /// will be a shortcut for that. I think I'll also need a client()
    /// that returns a pre-configured client that the plugin can use for
    /// more complex tasks.
    pub fn get(plugin: &dyn Plugin, url: &str) -> Result<Response> {
        let client = Client::builder()
            .https_only(true)
            .user_agent(super::plugin::USER_AGENT)
            .build()?;
        let request_token = PluginSupport::start_request(plugin, &url);
        let response = client.get(url).send()?;
        PluginSupport::end_request(request_token, 
            Some(response.status().as_u16()), 
            response.content_length());
        Ok(response)
    }
}

