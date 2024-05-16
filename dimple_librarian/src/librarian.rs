use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Model, Release, ReleaseGroup}
};

use anyhow::Result;

use crate::{merge::Merge, plugin::{NetworkMode, Plugin}};

#[derive(Clone)]
pub struct Librarian {
    db: Arc<SqliteDb>,
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
    network_mode: Arc<Mutex<NetworkMode>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).unwrap();
        let db_path = Path::new(path).join("dimple.db");
        let librarian = Self {
            db: Arc::new(SqliteDb::new(db_path.to_str().unwrap())),
            plugins: Default::default(),
            network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
        };
        librarian
    }

    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        plugin.init(self);
        plugin.set_network_mode(&self.network_mode());
        self.plugins.write().unwrap().push(plugin);
    }

    pub fn network_mode(&self) -> NetworkMode {
        self.network_mode.lock().unwrap().clone()
    }

    pub fn set_network_mode(&self, network_mode: &NetworkMode) {
        *self.network_mode.lock().unwrap() = network_mode.clone();
        for plugin in self.plugins.write().unwrap().iter_mut() {
            plugin.set_network_mode(network_mode);
        }
    }
}

impl Db for Librarian {
    fn insert(&self, model: &dimple_core::model::Model) -> Result<dimple_core::model::Model> {
        self.db.insert(model)
    }

    fn get(&self, model: &dimple_core::model::Model) -> Result<Option<dimple_core::model::Model>> {
        self.db.get(model)
    }

    fn link(&self, model: &dimple_core::model::Model, related_to: &dimple_core::model::Model) -> Result<()> {
        self.db.link(model, related_to)
    }

    fn list(
        &self,
        list_of: &dimple_core::model::Model,
        related_to: Option<&dimple_core::model::Model>,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        self.db.list(list_of, related_to)
    }
    
    fn reset(&self) -> Result<()> {
        self.db.reset()
    }
    
    /// Searches all plugins using the query string, filters the results down
    /// to artists, releases, and tracks, merges those results into the database
    /// and returns the merged, de-duplicated objects.
    fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        let mut iter = self.db.search(query)?;
        for plugin in self.plugins.read().unwrap().iter() {
            iter = Box::new(iter.chain(plugin.search(query, self.network_mode())?
                .map(|entity| entity.model())));
        }
        Ok(Box::new(iter))
    }
}
