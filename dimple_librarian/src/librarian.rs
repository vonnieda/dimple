use std::{collections::HashSet, fs, path::Path, sync::{Arc, Mutex, RwLock}, time::Instant};

use dimple_core::{
    db::{Db}, model::{Artist, Dimage, Entity, Model, ReleaseGroup}
};

use anyhow::{Error, Result};
use image::DynamicImage;

use crate::{merge::Merge, plugin::{NetworkMode, Plugin, PluginContext}, search, sqlite_db::SqliteDb};

// It's always worth reviewing https://www.subsonic.org/pages/api.jsp

#[derive(Clone)]
pub struct Librarian {
    db: Arc<Box<dyn Db>>,
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
    network_mode: Arc<Mutex<NetworkMode>>,
    plugin_context: PluginContext,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).unwrap();
        let db_path = Path::new(path).join("library.db");
        let plugin_cache_path = Path::new(path).join("plugin_cache");
        fs::create_dir_all(plugin_cache_path.clone()).unwrap();
        let librarian = Self {
            db: Arc::new(Box::new(SqliteDb::new(db_path.to_str().unwrap()).unwrap())),
            plugins: Default::default(),
            network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
            plugin_context: PluginContext::new(plugin_cache_path.to_str().unwrap()),
        };
        librarian
    }

    pub fn new_in_memory() -> Self {
        let librarian = Self {
            db: Arc::new(Box::new(SqliteDb::new(":memory:").unwrap())),
            plugins: Default::default(),
            network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
            plugin_context: PluginContext::default(),
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
            let results = plugin.search(query, self.network_mode(), &self.plugin_context);
            if let Ok(results) = results {
                for result in results {
                    self.merge(&result);
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

        // If we can find the model by key, merge it with the incoming one.
        if let Ok(Some(db_model)) = self.db.get(&model) {
            if let Some(merged) = Model::merge(model.clone(), db_model) {
                model = merged;
            }
        }

        let mut finished_plugins = HashSet::<String>::new();
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(plugin_model)) = plugin.get(&model.clone(), self.network_mode(), &self.plugin_context) {
                if let Some(merged) = Model::merge(model.clone(), plugin_model) {
                    model = merged;
                    finished_plugins.insert(plugin.name());
                }
            }
        }

        for plugin in self.plugins.read().unwrap().iter() {
            if finished_plugins.contains(&plugin.name()) {
                continue;
            }
            if let Ok(Some(plugin_model)) = plugin.get(&model.clone(), self.network_mode(), &self.plugin_context) {
                if let Some(merged) = Model::merge(model.clone(), plugin_model) {
                    model = merged;
                    finished_plugins.insert(plugin.name());
                }
            }
        }

        let result = self.merge(&model);
        
        Ok(Some(result))
    }

    pub fn plugin_cache_len(&self) -> usize {
        self.plugin_context.cache_len()
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
            let results = plugin.list(list_of, related_to, self.network_mode(), &self.plugin_context);
            if let Ok(results) = results {
                for result in results {
                    self.merge(&result);
                    if first {
                        return self.db.list(list_of, related_to)
                    }
                }
            }
        }

        self.db.list(list_of, related_to)
    }

    // TODO stopping here for now. Need to profile the whole image path and see
    // why it's so slow in dev mode. It's just unbearable. And maybe I really
    // just do need to use smaller images? Mik's album images are a good test.
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
        let t = Instant::now();
        let dimage = self._list(&Dimage::default().into(), &Some(model.clone()), true)
            .unwrap()
            .map(Into::<Dimage>::into)
            .next();
        if let Some(dimage) = dimage {
            log::debug!("image from plugins {:?} {}x{} in {}ms", dimage.key, dimage.width, 
                dimage.height, t.elapsed().as_millis());
            return Some(dimage.get_image())
        }

        // If nothing found specific to the model, see if there's something related.
        let t = Instant::now();
        match model {
            Model::Artist(artist) => {
                let release_groups = self.list2(ReleaseGroup::default(), Some(artist.clone()));
                if let Ok(release_groups) = release_groups {
                    for release_group in release_groups {
                        if let Some(dimage) = self.image(&release_group.model()) {
                            log::debug!("image from relations {}x{} in {}ms", dimage.width(), 
                                dimage.height(), t.elapsed().as_millis());
                            return Some(dimage)
                        }
                    }
                }
            },
            Model::Genre(genre) => {
                let release_groups = self.list2(ReleaseGroup::default(), Some(genre.clone()));
                if let Ok(release_groups) = release_groups {
                    for release_group in release_groups {
                        if let Some(dimage) = self.image(&release_group.model()) {
                            log::debug!("image from relations {}x{} in {}ms", dimage.width(), 
                                dimage.height(), t.elapsed().as_millis());
                            return Some(dimage)
                        }
                    }
                }
                let artists = self.list2(Artist::default(), Some(genre.clone()));
                if let Ok(artists) = artists {
                    for artist in artists {
                        if let Some(dimage) = self.image(&artist.model()) {
                            log::debug!("image from relations {}x{} in {}ms", dimage.width(), 
                                dimage.height(), t.elapsed().as_millis());
                            return Some(dimage)
                        }
                    }
                }
            }
            _ => ()
        }

        None
    }

    pub fn reset(&self) -> Result<()> {
        self.db.reset()
    } 

    /// Find, merges, and stores the given model in the database. Merges with
    /// existing data under the same key or identifier. On conflict, creates
    /// a duplicate. 
    pub fn merge(&self, model: &Model) -> Model {
        // start tx
        self.find_matching(model.clone())
            .and_then(|matching| Merge::merge(matching, model.clone()))
            .or_else(|| Some(model.clone()))
            .and_then(|merged| Some(self.store(merged)))
            .unwrap()
        // commit
    }

    /// Store the model under it's existing key, or under a new key if none.
    /// Returns the stored model, 
    fn store(&self, model: Model) -> Model {
        // TODO this is where db_merge would be, I guess, which would need to break up
        // the objects.
        // This is all clearly part of merge / librarian / "the api". This is
        // no different than a complex import process, which I do have some
        // experience with. Think of it like that - an import process.
        self.db.insert(&model).unwrap()
    }

    /// Finds a matching model to the one specified. Matching means that either
    /// a key or some other uniquely identifying information matches.
    fn find_matching(&self, model: Model) -> Option<Model> {
        // if artist, find_by_key, find_by_name_and_disambiguation, find_by_known_id
        // or one long SQl query with ORs
        // if release_group, find_by_artist_release_group?
        match model {
            Model::Artist(artist) => {
                self.db.list(&Artist::default().model(), &None).unwrap()
                    .map(Into::<Artist>::into)
                    .filter(|a| {
                        // SELECT * FROM Artist a
                        // WHERE 
                        //     (a.key IS NOT NULL AND a.key = ?)
                        //     OR (a.name IS NOT NULL AND a.name = ? AND a.disambiguation = ?)
                        //     OR (a.known_ids.musicbrainz_id = IS NOT NULL AND a.known_ids.musicbrainz_id = ?)
                        // ORDER BY a.name ASC

                        // self.db.lock().unwrap().query::<Artist, _>(
                        //     "
                        //     SELECT doc 
                        //     FROM Artist a
                        //     WHERE (a.key = ?) 
                        //     OR (a.doc->>'name' IS NOT NULL AND a.doc->>'name' = ? AND a.doc->>'disambiguation' = ?)
                        //     OR (a.doc->>'known_ids.musicbrainz_id' IS NOT NULL AND a.doc->>'known_ids.musicbrainz_id' = ?)
                        //     ", (artist.key, artist.name, artist.disambiguation, artist.known_ids.musicbrainz_id))
                        //     .unwrap().map(|a| a.model()).next()
        

                        (artist.key.is_some() && artist.key == a.key)
                        || (artist.name.is_some() && artist.name == a.name && artist.disambiguation == a.disambiguation)
                        || (artist.known_ids.musicbrainz_id.is_some() && artist.known_ids.musicbrainz_id == a.known_ids.musicbrainz_id)
                    })
                    .next()
                    .map(|a| a.model())
            },
            _ => todo!()
        }
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

    pub fn merge2<T: Entity + From<Model>>(&self, entity: T) -> T {
        self.merge(&entity.model()).into()
    }
}

