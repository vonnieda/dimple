use std::time::Instant;

use anyhow::Result;
use dimple_core::model::Entity;

use colored::Colorize;

pub const USER_AGENT: &str = "Dimple/0.0.1 +https://github.com/vonnieda/dimple +jason@vonnieda.org";

pub trait Plugin: Send + Sync {
    fn name(&self) -> String;

    /// Load the model using its key. Returns None if no key is set, or if the
    /// key doesn't exist in the database.
    fn get(&self, entity: &dyn Entity, network_mode: NetworkMode) -> Result<Option<Box<dyn Entity>>>;

    /// Get a list of models that are related to the specified model. If None is
    /// specified list all models of the specified type.
    fn list(
        &self,
        list_of: &dyn Entity,
        related_to: Option<&dyn Entity>,
        network_mode: NetworkMode,
    ) -> Result<Box<dyn Iterator<Item = Box<dyn Entity>>>>;

    fn search(&self, query: &str, network_mode: NetworkMode) 
        -> Result<Box<dyn Iterator<Item = Box<dyn Entity>>>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    Online,
    Offline,
}

pub struct LibrarySupport {
}

pub struct RequestToken {
    tag: String,
    start_time: Instant,
    url: String,
}

impl LibrarySupport {
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
}