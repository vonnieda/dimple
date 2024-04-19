use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::Picture, source::{AccessMode, Source}
};

use anyhow::Result;
use image::DynamicImage;

#[derive(Clone)]
pub struct Librarian {
    db: Arc<SqliteDb>,
    sources: Arc<RwLock<Vec<Box<dyn Source>>>>,
    access_mode: Arc<Mutex<AccessMode>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).unwrap();
        let db_path = Path::new(path).join("library.db");
        let librarian = Self {
            db: Arc::new(SqliteDb::new(db_path.to_str().unwrap())),
            sources: Default::default(),
            access_mode: Arc::new(Mutex::new(AccessMode::Online)),
        };
        librarian
    }

    pub fn add_source(&self, library: Box<dyn Source>) {
        self.sources.write().unwrap().push(library);
    }

    pub fn access_mode(&self) -> AccessMode {
        self.access_mode.lock().unwrap().clone()
    }

    pub fn set_access_mode(&self, value: &AccessMode) {
        *self.access_mode.lock().unwrap() = value.clone();
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
}

impl Source for Librarian {
    fn get(
        &self,
        model: &dimple_core::model::Model,
        _access_mode: &AccessMode,
    ) -> Result<Option<dimple_core::model::Model>> {
        self.db.get(model)
    }

    fn list(
        &self,
        list_of: &dimple_core::model::Model,
        related_to: Option<&dimple_core::model::Model>,
        _access_mode: &AccessMode,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        self.db.list(list_of, related_to)
    }
}
