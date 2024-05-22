use anyhow::Result;
use dimple_core::model::Entity;

pub trait Plugin: Send + Sync {
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
