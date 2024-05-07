use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Model}
};

use anyhow::Result;

use crate::plugin::{NetworkMode, Plugin};

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

    fn merge_artist(&self, artist: Artist) -> Artist {
        let artist_opt = self.list(&Artist::default().model(), None)
            .unwrap()
            .map(Into::<Artist>::into)
            .filter(|artist_opt| {
                if artist_opt.key.is_some() && artist_opt.key == artist.key {
                    true
                }
                else if artist_opt.known_ids.musicbrainz_id.is_some() 
                    && artist_opt.known_ids.musicbrainz_id == artist.known_ids.musicbrainz_id {
                    true
                }
                else if artist_opt.name.is_some() && artist_opt.name == artist.name {
                    true
                }
                else {
                    false
                }
            })
            .next();

        if let Some(mut artist_opt) = artist_opt {
            log::info!("merge ({:?}, {:?}, {:?}, {:?}) -> ({:?}, {:?}, {:?}, {:?})",
                artist.name,
                artist.country,
                artist.disambiguation,
                artist.known_ids.musicbrainz_id,
                artist_opt.name,
                artist_opt.country,
                artist_opt.disambiguation,
                artist_opt.known_ids.musicbrainz_id,
            );

            artist_opt.name = artist_opt.name.or(artist.name);
            artist_opt.country = artist_opt.country.or(artist.country);
            artist_opt.summary = artist_opt.summary.or(artist.summary);
            artist_opt.disambiguation = artist_opt.disambiguation.or(artist.disambiguation);
            artist_opt.known_ids.musicbrainz_id = artist_opt.known_ids.musicbrainz_id.or(artist.known_ids.musicbrainz_id);
            artist_opt.links = artist_opt.links.union(&artist.links).cloned().collect();
            
            self.insert(&artist_opt.model()).unwrap().into()
        }
        else {
            log::info!("insert new ({:?}, {:?}, {:?}, {:?})",
                artist.name,
                artist.country,
                artist.disambiguation,
                artist.known_ids.musicbrainz_id,
            );

            self.insert(&artist.model()).unwrap().into()
        }
    }

    pub fn merge(&self, model: Model) -> Model {
        match model {
            Model::Artist(artist) => self.merge_artist(artist).model(),
            _ => model,
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
    
    fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        self.db.search(query)
    }
}
