use std::sync::{Arc, Mutex, RwLock};

use dimple_core::{
    db::{Db, MemoryDb},
    source::{AccessMode, Source},
};

use anyhow::Result;

#[derive(Clone)]
pub struct Librarian {
    db: Arc<MemoryDb>,
    sources: Arc<RwLock<Vec<Box<dyn Source>>>>,
    access_mode: Arc<Mutex<AccessMode>>,
}

impl Librarian {
    pub fn new(_path: &str) -> Self {
        let librarian = Self {
            db: Arc::new(MemoryDb::default()),
            sources: Default::default(),
            access_mode: Arc::new(Mutex::new(AccessMode::Online)),
        };

        {
            // let librarian = librarian.clone();
            // thread::spawn(move || librarian.sync_worker());
        }

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
