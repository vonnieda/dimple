use std::{collections::HashSet, fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Model, ReleaseGroup}
};

use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{merge::{self, Merge}, plugin::{NetworkMode, Plugin}};


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

    // TODO Still struggling mightily with this, but I think it's worth the
    // effort to just go down this path, even if it gets thrown away, just to
    // get it working.
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

        self.local_library_search(query)
    }

    fn local_library_search(&self, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        const MAX_RESULTS_PER_TYPE: usize = 10;

        // TODO sort by score

        let pattern = query.to_string();
        let matcher = SkimMatcherV2::default();
        let artists = self.db.list(&Artist::default().model(), &None)?
            .filter(move |artist| {
                let artist: Artist = artist.clone().into();
                matcher.fuzzy_match(&artist.name.clone().unwrap_or_default(), &pattern).is_some()
            })
            .take(MAX_RESULTS_PER_TYPE);

        let pattern = query.to_string();
        let matcher = SkimMatcherV2::default();
        let release_groups = self.db.list(&ReleaseGroup::default().model(), &None)?
            .filter(move |rg| {
                let rg: ReleaseGroup = rg.clone().into();
                matcher.fuzzy_match(&rg.title.clone().unwrap_or_default(), &pattern).is_some()
            })
            .take(MAX_RESULTS_PER_TYPE);

        Ok(Box::new(artists.chain(release_groups)))
    }
}

impl Db for Librarian {
    /// Get a specific model using information (such as a key) in the
    /// specified model. The function first loads the model, if any,
    /// from the database and merges the input model. This model is
    /// then used to call get on each plugin, merging as it goes. Two passes
    /// are performed, allowing lower level plugins to supply additional
    /// data that higher level plugins can use.
    /// 
    /// If there are no results from storage or any plugin, returns Ok(None)
    fn get(&self, model: &Model) -> Result<Option<Model>> {
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

        Ok(self.merge(&model))
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        let db: &dyn Db = self.db.as_ref().as_ref();
        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.list(list_of, related_to, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    // TODO noting the use of db_merge_model here vs. self.merge in search
                    // because this asks for objects by relationship and thus needs to be
                    // merged with that same relationship.
                    merge::db_merge_model(db, &result, related_to);
                }
            }
        }

        self.db.list(list_of, related_to)
    }

    fn insert(&self, model: &dimple_core::model::Model) -> Result<dimple_core::model::Model> {
        self.db.insert(model)
    }

    fn link(&self, model: &dimple_core::model::Model, related_to: &dimple_core::model::Model) -> Result<()> {
        self.db.link(model, related_to)
    }
    
    fn reset(&self) -> Result<()> {
        self.db.reset()
    }
}

