use std::{collections::{HashMap, HashSet}, fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Genre, Model, Picture, ReleaseGroup, Track}
};

use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use image::DynamicImage;

use crate::{merge::{self, Merge}, plugin::{NetworkMode, Plugin}, search};


#[derive(Clone)]
pub struct Librarian {
    db: Arc<Box<dyn Db>>,
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
    network_mode: Arc<Mutex<NetworkMode>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).unwrap();
        let db_path = Path::new(path).join("dimple.db");
        let librarian = Self {
            db: Arc::new(Box::new(SqliteDb::new(db_path.to_str().unwrap()))),
            plugins: Default::default(),
            network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
        };
        librarian
    }

    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    }

    pub fn network_mode(&self) -> NetworkMode {
        self.network_mode.lock().unwrap().clone()
    }

    pub fn set_network_mode(&self, network_mode: &NetworkMode) {
        *self.network_mode.lock().unwrap() = network_mode.clone();
    }

    fn merge(&self, model: &Model) -> Option<Model> {
        merge::merge(self.db.clone().as_ref().as_ref(), model)
    }

    pub fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.search(query, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    self.merge(&result);
                }
            }
        }

        search::db_search(self.db.clone().as_ref().as_ref(), query)
    }

    /// Get a specific model using information (such as a key) in the
    /// specified model. The function first loads the model, if any,
    /// from the database and merges the input model. This model is
    /// then used to call get on each plugin, merging as it goes. Two passes
    /// are performed, allowing lower level plugins to supply additional
    /// data that higher level plugins can use.
    /// 
    /// If there are no results from storage or any plugin, returns Ok(None)
    pub fn get(&self, model: &Model) -> Result<Option<Model>> {
        let mut model = model.clone();

        if let Ok(Some(db_model)) = self.db.get(&model) {
            model = Model::merge(model, db_model);
        }

        let mut finished_plugins = HashSet::<String>::new();
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(plugin_model)) = plugin.get(&model, self.network_mode()) {
                model = Model::merge(model, plugin_model);
                finished_plugins.insert(plugin.name());
            }
        }

        for plugin in self.plugins.read().unwrap().iter() {
            if finished_plugins.contains(&plugin.name()) {
                continue;
            }
            if let Ok(Some(plugin_model)) = plugin.get(&model, self.network_mode()) {
                model = Model::merge(model, plugin_model);
            }
        }

        let result = self.merge(&model);

        Ok(result)
    }

    pub fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        let db: &dyn Db = self.db.as_ref().as_ref();

        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.list(list_of, related_to, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    merge::db_merge_model(db, &result, related_to);
                }
            }
        }

        let results = self.db.list(list_of, related_to);

        results
    }

    // TODO don't use list, go directly to plugins for the second phase
    // and abort when we get one.
    pub fn image(&self, model: &Model) -> Option<DynamicImage> {
        let picture = self.db.list(&Picture::default().into(), &Some(model.clone()))
            .unwrap()
            .map(Into::<Picture>::into)
            .next();
        if let Some(picture) = picture {
            return Some(picture.get_image())
        }

        let picture = self.list(&Picture::default().into(), &Some(model.clone()))
            .unwrap()
            .map(Into::<Picture>::into)
            .next();
        if let Some(picture) = picture {
            return Some(picture.get_image())
        }

        None
    }

    pub fn reset(&self) -> Result<()> {
        self.db.reset()
    } 
}

