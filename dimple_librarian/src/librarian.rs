use std::{collections::HashSet, fs, path::Path, sync::{Arc, Mutex, RwLock}, time::Instant};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Dimage, Entity, Model}
};

use anyhow::{Error, Result};
use image::DynamicImage;

use crate::{merge::{self, Merge}, plugin::{NetworkMode, Plugin}, search};

// It's always worth reviewing https://www.subsonic.org/pages/api.jsp

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

    pub fn new_in_memory() -> Self {
        let librarian = Self {
            db: Arc::new(Box::new(SqliteDb::new(":memory:"))),
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

    pub fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = Model>>> {
        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.search(query, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    merge::merge(self.db.as_ref().as_ref(), &result, &None);
                }
            }
        }

        search::db_search(self.db.as_ref().as_ref(), query)
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

        let result = merge::merge(self.db.as_ref().as_ref(), &model, &None);
        
        Ok(result)
    }

    pub fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        self._list(list_of, related_to, false)
    }

    fn _list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
        first: bool,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        let db: &dyn Db = self.db.as_ref().as_ref();

        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.list(list_of, related_to, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    merge::merge(db, &result, related_to);
                    if first {
                        return self.db.list(list_of, related_to)
                    }
                }
            }
        }

        self.db.list(list_of, related_to)
    }

    // TODO don't use list, go directly to plugins for the second phase
    // and abort when we get one.
    pub fn image(&self, model: &Model) -> Option<DynamicImage> {
        let t = Instant::now();
        let dimage = self.db.list(&Dimage::default().into(), &Some(model.clone()))
            .unwrap()
            .map(Into::<Dimage>::into)
            .next();
        if let Some(dimage) = dimage {
            log::debug!("image from database {:?} {}x{} in {}ms", dimage.key, dimage.width, 
                dimage.height, t.elapsed().as_millis());
            return Some(dimage.get_image())
        }

        // TODO note, this uses a specialization of list that returns on the 
        // first valid result to speed things up. Eventually I want Dimage to
        // not include the blob, and then this won't be needed, or wanted,
        // because we'll want to be able to offer the user all the different
        // images, not just one.
        let dimage = self._list(&Dimage::default().into(), &Some(model.clone()), true)
            .unwrap()
            .map(Into::<Dimage>::into)
            .next();
        if let Some(dimage) = dimage {
            log::debug!("image from plugins {:?} {}x{} in {}ms", dimage.key, dimage.width, 
                dimage.height, t.elapsed().as_millis());
            return Some(dimage.get_image())
        }

        None
    }

    pub fn merge(&self, model: &Model, related_to: &Option<Model>) -> Option<Model> {
        merge::merge(self.db.clone().as_ref().as_ref(), model, related_to)
    }

    pub fn reset(&self) -> Result<()> {
        self.db.reset()
    } 

    // Playing around with using a little bit of generic sugar to make these
    // APIs much more ergonomic. 
    pub fn get2<T: Entity + std::convert::From<Model>>(&self, entity: T) -> Result<T> {
        let result = self.get(&entity.model())?;
        if result.is_none() {
            return Err(Error::msg("not found"))
        }
        Ok(result.unwrap().into())
    }

    pub fn list2<T, N> (
        &self,
        list_of: T,
        related_to: Option<N>,
    ) 
    -> Result<Box<dyn Iterator<Item = T>>>
    where 
        T: Entity + std::convert::From<Model> + 'static,
        N: Entity + std::convert::From<Model> {

        let a = self.list(&list_of.model(), &related_to.map(|r| r.model()))?;
        Ok(Box::new(a.map(Into::<T>::into)))
    }
}

#[cfg(test)]
mod test {
    use dimple_core::model::{Artist, Entity, KnownIds, Model};

    use crate::plugin::Plugin;

    use super::Librarian;

    #[test]
    fn merge_basics() {
        let lib = Librarian::new_in_memory();
        lib.add_plugin(Box::new(TestPlugin::default()));
        let artist: Artist = lib.get(&Artist {
            known_ids: KnownIds {
                musicbrainz_id: Some("DIMPLE-TEST-METALLICA".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }.model()).unwrap().unwrap().into();
        assert!(artist.name == Some("Metallica".to_string()));
        dbg!(&artist);
    }

    #[derive(Default)]
    struct TestPlugin {

    }

    impl Plugin for TestPlugin {
        fn name(&self) -> String {
            "Test".to_string()
        }
        
        fn get(&self, model: &dimple_core::model::Model, network_mode: crate::plugin::NetworkMode) -> anyhow::Result<Option<dimple_core::model::Model>> {
            match model {
                Model::Artist(artist) => {
                    if artist.known_ids.musicbrainz_id == Some("DIMPLE-TEST-METALLICA".to_string()) {
                        return Ok(Some(Artist {
                            name: Some("Metallica".to_string()),
                            summary: Some("Metal band from LA".to_string()),
                            ..Default::default()
                        }.model()));
                    }
                },
                _ => ()
            }          
            Ok(None)  
        }
    }
}