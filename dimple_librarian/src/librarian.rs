use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, MemoryDb, SqliteDb}, model::{self, Artist, Entity, Genre, Medium, Model, Recording, Release, ReleaseGroup, Track}
};

use anyhow::Result;

use crate::{merge::Merge, plugin::{NetworkMode, Plugin}};

use rayon::prelude::{*};

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

    // TODO last thing to decide here is how we're store the sub-objects
    // in the db.
    fn merge(&self, model: &Model) -> Option<Model> {
        match model {
            Model::Artist(artist) => self.merge_artist(artist),
            Model::Release(release) => self.merge_release(release),
            Model::ReleaseGroup(release_group) => self.merge_release_group(release_group),
            Model::Recording(recording) => self.merge_recording(recording),
            _ => todo!(),
        }
    }

    fn merge_artist(&self, artist: &Artist) -> Option<Model> {
        Self::db_merge_model(self, &artist.model(), &None)
    }

    fn merge_release_group(&self, release_group: &ReleaseGroup) -> Option<Model> {
        Self::db_merge_model(self, &release_group.model(), &None)
    }

    fn merge_release(&self, release: &Release) -> Option<Model> {
        Self::db_merge_model(self, &release.model(), &None)
    }

    fn merge_recording(&self, recording: &Recording) -> Option<Model> {
        None
    }

    fn db_merge_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
        // find a matching model to the specified, merge, save
        let matching = Self::find_matching_model(db, model, parent);
        if let Some(matching) = matching {
            let merged = Self::merge_model(&model, &matching);
            return Some(db.insert(&merged).unwrap())
        }
        // if not, insert the new one and link it to the parent
        else {
            if Self::model_valid(model) {
                let model = Some(db.insert(model).unwrap());
                Self::lazy_link(db, &model, parent);
                return model
            }
        }
        None
    }

    /// Links the two Models if they are both Some. Reduces boilerplate.
    fn lazy_link(db: &dyn Db, l: &Option<Model>, r: &Option<Model>) {
        if l.is_some() && r.is_some() {
            db.link(&l.clone().unwrap(), &r.clone().unwrap()).unwrap()
        }
    }

    fn find_matching_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
        // This needs to use the scoring and sorting, but yea.
        db.list(&model, parent.as_ref()).unwrap()
            .find(|model_opt| Self::compare_models(&model, model_opt))
    }

    fn compare_models(l: &Model, r: &Model) -> bool {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => {
                (l.name.is_some() && l.name == r.name && l.disambiguation == r.disambiguation)
                || (l.known_ids.musicbrainz_id.is_some() && l.known_ids.musicbrainz_id == r.known_ids.musicbrainz_id)
            },
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => {
                l.title.is_some() && l.title == r.title
            },
            (Model::Release(l), Model::Release(r)) => {
                l.title.is_some() && l.title == r.title
            },
            (Model::Medium(l), Model::Medium(r)) => {
                l.position == r.position
            },
            (Model::Track(l), Model::Track(r)) => {
                l.title.is_some() && l.title == r.title
            },
            (Model::Genre(l), Model::Genre(r)) => {
                l.name.is_some() && l.name == r.name
            },
            _ => todo!()
        }
    }

    fn merge_model(l: &Model, r: &Model) -> Model {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => Artist::merge(l.clone(), r.clone()).model(),
            (Model::Genre(l), Model::Genre(r)) => Genre::merge(l.clone(), r.clone()).model(),
            (Model::Medium(l), Model::Medium(r)) => Medium::merge(l.clone(), r.clone()).model(),
            (Model::Release(l), Model::Release(r)) => Release::merge(l.clone(), r.clone()).model(),
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => ReleaseGroup::merge(l.clone(), r.clone()).model(),
            (Model::Track(l), Model::Track(r)) => Track::merge(l.clone(), r.clone()).model(),
            _ => todo!()
        }
    }

    fn model_valid(model: &Model) -> bool {
        match model {
            Model::Artist(a) => a.name.is_some() || a.known_ids.musicbrainz_id.is_some(),
            Model::Genre(g) => g.name.is_some(),
            Model::Medium(_m) => true,
            Model::Release(r) => r.title.is_some(),
            Model::ReleaseGroup(rg) => rg.title.is_some(),
            Model::Track(t) => t.title.is_some(),
            _ => todo!()
        }
    }
}

impl Db for Librarian {
    fn insert(&self, model: &dimple_core::model::Model) -> Result<dimple_core::model::Model> {
        self.db.insert(model)
    }

    fn get(&self, model: &dimple_core::model::Model) -> Result<Option<dimple_core::model::Model>> {
        let model = self.db.get(model)?.unwrap_or(model.clone());

        for plugin in self.plugins.read().unwrap().iter() {
            // TODO I don't think this should return on a plugin error, as
            // we can still go local.
            if let Some(result) = plugin.get(model.entity(), self.network_mode())? {
                self.merge(&result.model());
            }
        }

        self.db.get(&model)
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
        for plugin in self.plugins.read().unwrap().iter() {
            // TODO I don't think this should return on a plugin error, as
            // we can still go local.
            for result in plugin.search(query, self.network_mode())? {
                self.merge(&result.model());
            }
        }
        self.db.search(query)
    }
}


// use std::{collections::HashSet, sync::{Mutex, RwLock}};

// use dimple_core::{collection::Collection, model::{Artist, Recording, RecordingSource, Release, ReleaseGroup}};
// use dimple_sled_library::sled_library::SledLibrary;
// use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
// use image::DynamicImage;
// use rayon::prelude::*;
// use dimple_core::model::Entities;

// use crate::{art_gen, matching};

// pub struct Librarian {
//     local_library: SledLibrary,
//     libraries: RwLock<Vec<Box<dyn Collection>>>,
//     access_mode: Mutex<AccessMode>,
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum AccessMode {
//     Online,
//     Offline,
// }

// impl Librarian {
//     pub fn new(path: &str) -> Self {
//         Self { 
//             local_library: SledLibrary::new(path),
//             libraries: Default::default(),
//             access_mode: Mutex::new(AccessMode::Online),
//         }
//     }

//     pub fn add_library(&self, library: Box<dyn Collection>) {
//         self.libraries.write().unwrap().push(library);
//     }

//     pub fn access_mode(&self) -> AccessMode {
//         self.access_mode.lock().unwrap().clone()
//     }

//     pub fn set_access_mode(&self, value: &AccessMode) {
//         *self.access_mode.lock().unwrap() = value.clone();
//     }

//     /// Get or create a thumbnail image at the given size for the entity.
//     /// If no image can be loaded from the library one is generated. Results
//     /// either from the library or generated are cached for future calls.
//     pub fn thumbnail(&self, entity: &Entities, width: u32, height: u32) -> DynamicImage {
//         let cached = self.local_library.images.get(&entity.key().unwrap(), width, height);
//         if let Some(dyn_image) = cached {
//             return dyn_image;
//         }
//         else if let Some(dyn_image) = self.image(entity) {
//             self.local_library.set_image(entity, &dyn_image);
//             return self.local_library.images.get(&entity.key().unwrap(), width, height).unwrap();
//         }
//         let generated = art_gen::generate_masterpiece(self, entity, width, height);
//         self.local_library.set_image(entity, &generated);
//         self.local_library.images.get(&entity.key().unwrap(), width, height).unwrap()
//     }

//     /// Find a matching model in the local store merge the value, and store it. 
//     /// If no match is found a new model is stored and returned.
//     /// TODO need to think about how this merges after an offline session, and
//     /// I think that moves it more towards a reusable component that happens
//     /// in the foreground when needed but mostly in the background.
//     /// TODO Okay, I think this is going to start storing every entity, matching
//     /// maybe only on source id, and then what the UI will get will be merged in
//     /// real time? Or also merged at the same time, but primarily gonna start
//     /// storing and being able to reference all the source objects.
//     /// Oh wow, and then I could just have the queries run on the dimple source
//     /// ids. Or even librarian.
//     pub fn merge(&self, model: &Entities, related_to: Option<&Entities>) -> Entities {
//         // TODO this needs to be atomic. When using par_iter it blows up cause
//         // we're saving out of date info.
//         let matched = matching::find_match(self, &self.local_library, model);
//         let merged = match matched {
//             Some(matched) => Entities::merge(matched, model.clone()),
//             None => model.clone(),
//         };
//         let saved = self.local_library.set(&merged).unwrap();
//         if let Some(related_to) = related_to {
//             let related_to = self.merge(related_to, None);
//             let _ = self.local_library.link(&saved, &related_to, "by");
//         }
//         saved
//     }

//     fn local_library_search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
//         // const MAX_RESULTS_PER_TYPE: usize = 10;
//         // TODO sort by score
//         // TODO other entities
//         let pattern = query.to_string();
//         let matcher = SkimMatcherV2::default();
//         let artists = Artist::list(&self.local_library)
//             .filter(move |a| matcher.fuzzy_match(&a.name.clone().unwrap_or_default(), &pattern).is_some())
//             .map(Entities::Artist);
//             // .take(MAX_RESULTS_PER_TYPE);
//         let pattern = query.to_string();
//         let matcher = SkimMatcherV2::default();
//         let release_groups = ReleaseGroup::list(&self.local_library)
//             .filter(move |a| matcher.fuzzy_match(&a.title.clone().unwrap_or_default(), &pattern).is_some())
//             .map(Entities::ReleaseGroup);
//             // .take(MAX_RESULTS_PER_TYPE);
//         Box::new(artists.chain(release_groups))
//     }
// }

// impl Collection for Librarian {
//     fn name(&self) -> String {
//         "Librarian".to_string()
//     }

//     /// Search the libraries, merge the results to local, and then search
//     /// local and return the results. This merge step ensures that the objects
//     /// returned are the sum of the sources.
//     fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
//         if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
//             self.libraries.read().unwrap().iter()
//                 .flat_map(|lib| lib.search(query))
//                 .for_each(|m| {
//                     let _ = self.merge(&m, None);
//                 });
//         }
//         self.local_library_search(query)
//     }    

//     /// List the libraries, merge the results to local, and then list
//     /// local and return the results. This merge step ensures that the objects
//     /// returned are the sum of the sources.
//     fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
//         // TODO parallel version is much faster but self.merge is not atomic
//         // But note, the real problem is the "list" in the match function.
//         if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
//             self.libraries.read().unwrap().iter()
//                 .flat_map(|lib| lib.list(of_type, related_to))
//                 .for_each(|m| {
//                     let _ = self.merge(&m, related_to);
//                 });
//         }
//         self.local_library.list(of_type, related_to)
//     }

//     /// First, merge the request entity as a side effect. Search the libraries, 
//     /// merge the results to local, and then search local and return the 
//     /// results. This merge step ensures that the objects returned are the 
//     /// sum of the sources.
//     fn fetch(&self, entity: &Entities) -> Option<Entities> {
//         // Merging the entity before searching ensures we have things like
//         // additional ids for querying. I'm not totally sure if this should
//         // merge (to disk) or if it should query and merge in memory for
//         // the query only. Since we're gonna save it at the end anyway I don't
//         // see any harm in saving it here too. This could be replaced with
//         // find_match and merge in memory though.

//         // TODO I don't think this logic is totally sound. At the end we fetch
//         // using this entity, but what if we've merged something? I think we
//         // need to replace the query with the _ at the end of the sub-block.
//         // This may possibly explain the weird bug where Spidergawd shows up in
//         // Opeth

//         let entity = self.merge(entity, None);

//         if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
//             // Run the fetch on all of the libraries, keeping track of the ones
//             // that return a result. 
//             let skip_libs: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
//             let first_result: Entities = self.libraries.read().ok()?.par_iter()
//                 .filter_map(|lib| {
//                     let result = lib.fetch(&entity);
//                     if result.is_some() {
//                         skip_libs.lock().unwrap().insert(lib.name());
//                     }
//                     result
//                 })
//                 .reduce(|| entity.clone(), Entities::merge);
            
//             // Run the fetch on the remaining libraries that did not return a result
//             // the first time. This allows libraries that need metadata from
//             // for instance, Musicbrainz to skip the first fetch and run on
//             // this one instead.
//             let second_result: Entities = self.libraries.read().ok()?.par_iter()
//                 .filter(|f| !skip_libs.lock().unwrap().contains(&f.name()))
//                 .filter_map(|lib| lib.fetch(&first_result))
//                 .reduce(|| entity.clone(), Entities::merge);

//             // Merge the results together and store it for later access
//             let result = Entities::merge(first_result, second_result);
//             let _ = self.merge(&result, None);
//         }

//         // Return the results from the local library
//         self.local_library.fetch(&entity)
//     }

//     /// If there is an image stored in the local library for the entity return
//     /// it, otherwise search the attached libraries for one. If one is found,
//     /// cache it in the local library and return it. Future requests will be
//     /// served from the cache.
//     /// TODO I think this goes away and becomes fetch(Dimage), maybe Dimage
//     /// so it's not constantly overlapping with Image.
//     fn image(&self, entity: &Entities) -> Option<DynamicImage> {
//         let image = self.local_library.image(entity);
//         if image.is_some() {
//             return image;
//         }

//         if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
//             if let Some(dyn_image) = self.libraries.read().unwrap().iter().find_map(|lib| lib.image(entity)) {
//                 self.local_library.set_image(entity, &dyn_image);
//                 return Some(dyn_image)
//             }
//         }
//         None
//     }

//     fn stream(&self, entity: &Entities) -> Option<Box<dyn Iterator<Item = u8>>> {
//         let iter = self.local_library.stream(entity);
//         if iter.is_some() {
//             log::info!("found local");
//             return iter;
//         }

//         if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
//             if let Some(iter) = self.libraries.read().unwrap().iter().find_map(|lib| lib.stream(entity)) {
//                 log::info!("found in lib");
//                 // self.local_library.set_image(entity, &dyn_image);
//                 return Some(iter)
//             }
//         }
//         None
//     }    
// }
