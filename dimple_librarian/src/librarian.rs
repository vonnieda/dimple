use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Model, ReleaseGroup}
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

    fn merge_artist(&self, artist: Artist) -> Option<Model> {
        // TODO if there's no identifying information, what do we do? I think
        // we need to not create the entity, and it's links. I think that works
        // for basically everything.
        // TODO this can be genericified on Entity + Model 

        let mut artists: Vec<(Artist, f32)> = self.list(&Artist::default().model(), None)
            .unwrap()
            .map(Into::<Artist>::into)
            .map(|artist_opt| (artist_opt.clone(), Artist::mergability(&artist, &artist_opt)))
            .filter(|(_artist_opt, score)| *score >= 1.0)
            .collect();
        artists.sort_by(|(_artist_l, score_l), (_artist_r, score_r)| score_l.partial_cmp(score_r).unwrap());

        if let Some((artist_opt, _score)) = artists.get(0) {
            let merged = Artist::merge(artist_opt.clone(), artist.clone());
            Some(self.insert(&merged.model()).unwrap().into())
        }
        else {
            Some(self.insert(&artist.model()).unwrap().into())
        }
    }

    fn merge_release_group(&self, model: ReleaseGroup) -> Option<Model> {
        // TODO if there's no identifying information, what do we do? I think
        // we need to not create the entity, and it's links. I think that works
        // for basically everything.
        // TODO this can be genericified on Entity + Model 

        let mut options: Vec<(ReleaseGroup, f32)> = self.list(&model.model(), None)
            .unwrap()
            .map(Into::<ReleaseGroup>::into)
            .map(|option| (option.clone(), ReleaseGroup::mergability(&model, &option)))
            .filter(|(_option, score)| *score >= 1.0)
            .collect();
        options.sort_by(|(_option_l, score_l), (_option_r, score_r)| score_l.partial_cmp(score_r).unwrap());

        if let Some((option, _score)) = options.get(0) {
            let merged = ReleaseGroup::merge(option.clone(), model.clone());
            Some(self.insert(&merged.model()).unwrap().into())
        }
        else {
            if model.title.is_none() {
                return None
            }
            Some(self.insert(&model.model()).unwrap().into())
        }
    }

    pub fn merge(&self, model: Model) -> Option<Model> {
        match model {
            Model::Artist(artist) => self.merge_artist(artist),
            Model::ReleaseGroup(release_group) => self.merge_release_group(release_group),
            _ => todo!(),
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
