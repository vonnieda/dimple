use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Release}, source::{AccessMode, Source}
};

use anyhow::Result;

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

        let db = librarian.db.clone();

        let artist1: Artist = db
        .insert(
            &Artist {
                name: Some("Rick and Morty".to_string()),
                ..Default::default()
            }
            .into(),
        ).unwrap()
        .into();

    let artist2: Artist = db
        .insert(
            &Artist {
                name: Some("Infected Mushroom".to_string()),
                ..Default::default()
            }
            .into(),
        ).unwrap()
        .into();

    let artist3: Artist = db
        .insert(
            &Artist {
                name: Some("Hoodie Poo".to_string()),
                ..Default::default()
            }
            .into(),
        ).unwrap()
        .into();

    let release1: Release = db
        .insert(
            &Release {
                title: Some("Mega Seeds".to_string()),
                ..Default::default()
            }
            .into(),
        ).unwrap()
        .into();

    let release2: Release = db
        .insert(
            &Release {
                title: Some("Boss La Rosh".to_string()),
                ..Default::default()
            }
            .into(),
        ).unwrap()
        .into();

    let release3: Release = db
        .insert(
            &Release {
                title: Some("All Together Now".to_string()),
                ..Default::default()
            }
            .into(),
        ).unwrap()
        .into();

    db.link(&release1.clone().into(), &artist1.clone().into()).unwrap();
    db.link(&release2.clone().into(), &artist2.clone().into()).unwrap();
    db.link(&release3.clone().into(), &artist1.clone().into()).unwrap();
    db.link(&release3.clone().into(), &artist2.clone().into()).unwrap();
    db.link(&release3.clone().into(), &artist3.clone().into()).unwrap();
        


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
