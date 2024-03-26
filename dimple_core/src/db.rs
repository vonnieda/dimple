use anyhow::Result;

use crate::model::Model;

mod memory_db;
mod sqlite_db;

pub use memory_db::MemoryDb;
pub use sqlite_db::SqliteDb;

pub trait Db: Send + Sync {
    /// Save the model in the database using its key. If no key is set, create
    /// a unique one, save the model with it, and return the model with the
    /// new key set. Overwrites existing values.
    fn insert(&self, model: &Model) -> Result<Model>;

    /// Load the model using its key. Returns None if no key is set, or if the
    /// key doesn't exist in the database.
    fn get(&self, model: &Model) -> Result<Option<Model>>;

    // /// Like insert() but TODO takes a merge function and performs a get,
    // /// merge, update in a transaction.
    // fn merge(&self, model: &Model) -> Result<Model>;

    /// Link two models in a many to many relationship such that it is possible
    /// to retrieve a list of models related to the specified model.
    fn link(&self, model: &Model, related_to: &Model) -> Result<()>;

    /// Get a list of models that are related to the specified model. If None is
    /// specified list all models of the specified type.
    fn list(
        &self,
        list_of: &Model,
        related_to: Option<&Model>,
    ) -> Result<Box<dyn Iterator<Item = Model>>>;
}
