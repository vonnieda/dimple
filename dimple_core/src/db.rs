use anyhow::Result;

use crate::model::{Artist, Entity, Model};

mod sqlite_db;

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

    fn reset(&self) -> Result<()>;
}

pub trait EntityDb: Send + Sync {
    fn insert(&self, entity: &dyn Entity) -> Result<Box<dyn Entity>>;

    fn get(&self, entity: &dyn Entity) -> Result<Option<Box<dyn Entity>>>;

    fn link(&self, entity: &dyn Entity, related_to: &dyn Entity) -> Result<()>;

    fn list(
        &self,
        list_of: &dyn Entity,
        related_to: Option<&dyn Entity>,
    ) -> Result<Box<dyn Iterator<Item = Box<dyn Entity>>>>;
}

struct Temp {

}

impl Temp {
    pub fn do_stuff(db: &dyn EntityDb) {
        let artist = Artist::default();
        let artist = db.insert(&artist).unwrap();
        let artist = artist.as_any().downcast_ref::<Artist>().unwrap();
        let artists: Vec<_> = db.list(&Artist::default(), None).unwrap().map(|a| a.as_any().downcast_ref::<Artist>().unwrap().clone()).collect();
    }
}
