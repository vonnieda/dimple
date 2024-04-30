use std::path::PathBuf;

use anyhow::Result;

use crate::librarian::Librarian;

pub trait Plugin: Send + Sync {
    // /// Load the model using its key. Returns None if no key is set, or if the
    // /// key doesn't exist in the database.
    // fn get(&self, model: &Model, network_mode: &NetworkMode) -> Result<Option<Model>>;

    // /// Get a list of models that are related to the specified model. If None is
    // /// specified list all models of the specified type.
    // fn list(
    //     &self,
    //     list_of: &Model,
    //     related_to: Option<&Model>,
    //     network_mode: &NetworkMode,
    // ) -> Result<Box<dyn Iterator<Item = Model>>>;


    // TODO gonna change this up, hate the init thing
    fn init(&self, librarian: &Librarian);
    // TODO won't need this because will always have librarian, and can check there
    fn set_network_mode(&self, _network_mode: &NetworkMode);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    Online,
    Offline,
}
