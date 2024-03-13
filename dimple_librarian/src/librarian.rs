use std::{collections::HashSet, sync::{Arc, Mutex, RwLock}, thread, time::Duration};

use dimple_core::{collection::Collection, model::{Artist, Entity, MediaFile, Release, ReleaseGroup}};
use dimple_sled_library::sled_library::SledLibrary;
use image::DynamicImage;
use dimple_core::model::Entities;

use crate::{art_gen, merge::{self, Merge}};

#[derive(Clone)]
pub struct Librarian {
    local_library: Arc<SledLibrary>,
    libraries: Arc<RwLock<Vec<Box<dyn Collection>>>>,
    access_mode: Arc<Mutex<AccessMode>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessMode {
    Online,
    Offline,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        let librarian = Self { 
            local_library: Arc::new(SledLibrary::new(path)),
            libraries: Default::default(),
            access_mode: Arc::new(Mutex::new(AccessMode::Online)),
        };

        {
            let librarian = librarian.clone();
            thread::spawn(move || librarian.metadata_worker());
        }

        {
            let librarian = librarian.clone();
            thread::spawn(move || librarian.coverart_worker());
        }

        librarian
    }

    /**
     * Downloads and stores coverart for entities that do not yet have any.
     */
    fn coverart_worker(&self) {
        loop {
            thread::sleep(Duration::from_secs(10));
        }
    }

    /**
     * Imports MediaFile entities from libraries that supply them.
     */
    fn import_worker(&self) {
        loop {
            thread::sleep(Duration::from_secs(10));
            let access_mode = self.access_mode();
            for lib in self.libraries.read().unwrap().iter() {
                if access_mode == AccessMode::Online || lib.available_offline() {
                    // TODO MediaFiles
                    let media_files: Vec<_> = MediaFile::list(lib.as_ref()).collect();
                    log::info!("merging {}", media_files.len());
                    for media_file in media_files {
                        self.local_library.set(&media_file.entity()).unwrap();
                    }
                }
            }
            log::info!("merged em shits");
        }
    }
    
    /**
     * Improves metadata for stored entities over time by merging in additional
     * details as they are found.
     */
    fn metadata_worker(&self) {
        loop {
            thread::sleep(Duration::from_secs(10));
        }
    }

    pub fn add_library(&self, library: Box<dyn Collection>) {
        self.libraries.write().unwrap().push(library);
    }

    pub fn access_mode(&self) -> AccessMode {
        self.access_mode.lock().unwrap().clone()
    }

    pub fn set_access_mode(&self, value: &AccessMode) {
        *self.access_mode.lock().unwrap() = value.clone();
    }

    /// Get or create a thumbnail image at the given size for the entity.
    /// If no image can be loaded from the library one is generated. Results
    /// either from the library or generated are cached for future calls.
    pub fn thumbnail(&self, entity: &Entities, width: u32, height: u32) -> DynamicImage {
        let cached = self.local_library.images.get(&entity.key().unwrap(), width, height);
        if let Some(dyn_image) = cached {
            return dyn_image;
        }
        else if let Some(dyn_image) = self.image(entity) {
            self.local_library.set_image(entity, &dyn_image);
            return self.local_library.images.get(&entity.key().unwrap(), width, height).unwrap();
        }
        let generated = art_gen::generate_masterpiece(self, entity, width, height);
        self.local_library.set_image(entity, &generated);
        self.local_library.images.get(&entity.key().unwrap(), width, height).unwrap()
    }

    /// Find a matching model in the local store merge the value, and store it. 
    /// If no match is found a new model is stored and returned.
    // TODO this needs to be atomic. When using par_iter it blows up cause
    // we're saving out of date info.
    fn merge(&self, model: &Entities, related_to: Option<&Entities>) -> Entities {
        let matched = self.list(model, None).find(|option| Entities::mergability(model, option) >= 1.0);
        let merged = match matched {
            Some(matched) => Entities::merge(matched, model.clone()),
            None => model.clone(),
        };
        let saved = self.local_library.set(&merged).unwrap();
        if let Some(related_to) = related_to {
            let related_to = self.merge(related_to, None);
            let _ = self.local_library.link(&saved, &related_to, "by");
        }
        saved
    }

    // pub fn find_match(librarian: &Librarian, local_library: &SledLibrary, model: &Entities) -> Option<Entities> {
    //     // Find by key
    //     if let Some(model) = local_library.get(model) {
    //         return Some(model)
    //     }

    //     // Find by source id, known id, or fuzzy match
    //     for candidate in local_library.list(model, None) {
    //         let l = model;
    //         let r = m;
    //         if Entities::mergability(l, &r) > 0.75 {
    //             return Some(m)
    //         }
    //     }

    //     // Give up
    //     None
    // }

    fn local_library_search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
        todo!();
        // // const MAX_RESULTS_PER_TYPE: usize = 10;
        // // TODO sort by score
        // // TODO other entities
        // let pattern = query.to_string();
        // let matcher = SkimMatcherV2::default();
        // let artists = Artist::list(self.local_library.as_ref())
        //     .filter(move |a| matcher.fuzzy_match(&a.name.clone().unwrap_or_default(), &pattern).is_some())
        //     .map(Entities::Artist);
        //     // .take(MAX_RESULTS_PER_TYPE);
        // let pattern = query.to_string();
        // let matcher = SkimMatcherV2::default();
        // let release_groups = ReleaseGroup::list(self.local_library.as_ref())
        //     .filter(move |a| matcher.fuzzy_match(&a.title.clone().unwrap_or_default(), &pattern).is_some())
        //     .map(Entities::ReleaseGroup);
        //     // .take(MAX_RESULTS_PER_TYPE);
        // Box::new(artists.chain(release_groups))
    }
}

// struct MergingLibrarian(Librarian);
// impl Collection for MergingLibrarian {
//     fn name(&self) -> String {
//         "Librarian".to_string()
//     }

    
//     /// Search the libraries, merge the results to local, and then search
//     /// local and return the results. This merge step ensures that the objects
//     /// returned are the sum of the sources.
//     fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
//         let access_mode = self.access_mode();
//         self.libraries.read().unwrap().iter()
//             .filter(|lib| access_mode == AccessMode::Online || lib.available_offline())
//             .flat_map(|lib| lib.search(query))
//             .for_each(|m| {
//                 let _ = self.merge(&m, None);
//             });
//         self.local_library_search(query)
//     }    

//     /// List the libraries, merge the results to local, and then list
//     /// local and return the results. This merge step ensures that the objects
//     /// returned are the sum of the sources.
//     /// TODO parallel version is much faster but self.merge is not atomic
//     /// But note, the real problem is the "list" in the match function.
//     fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
//         let access_mode = self.access_mode();
//         self.libraries.read().unwrap().iter()
//             .filter(|lib| access_mode == AccessMode::Online || lib.available_offline())
//             .flat_map(|lib| lib.list(of_type, related_to))
//             .for_each(|m| {
//                 let _ = self.merge(&m, related_to);
//             });
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

//         let access_mode = self.access_mode();

//         // Run the fetch on all of the libraries, keeping track of the ones
//         // that return a result. 
//         let skip_libs: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
//         let first_result: Entities = self.libraries.read().ok()?.par_iter()
//             .filter(|lib| access_mode == AccessMode::Online || lib.available_offline())
//             .filter_map(|lib| {
//                 let result = lib.fetch(&entity);
//                 if result.is_some() {
//                     skip_libs.lock().unwrap().insert(lib.name());
//                 }
//                 result
//             })
//             .reduce(|| entity.clone(), Entities::merge);
            
//         // Run the fetch on the remaining libraries that did not return a result
//         // the first time. This allows libraries that need metadata from
//         // for instance, Musicbrainz to skip the first fetch and run on
//         // this one instead.
//         let second_result: Entities = self.libraries.read().ok()?.par_iter()
//             .filter(|lib| !skip_libs.lock().unwrap().contains(&lib.name()))
//             .filter(|lib| access_mode == AccessMode::Online || lib.available_offline())
//             .filter_map(|lib| lib.fetch(&first_result))
//             .reduce(|| entity.clone(), Entities::merge);

//         // Merge the results together and store it for later access
//         let result = Entities::merge(first_result, second_result);
//         let _ = self.merge(&result, None);

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
//             return iter;
//         }

//         if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
//             if let Some(iter) = self.libraries.read().unwrap().iter().find_map(|lib| lib.stream(entity)) {
//                 // self.local_library.set_image(entity, &dyn_image);
//                 return Some(iter)
//             }
//         }
//         None
//     }    
// }

impl Collection for Librarian {
    fn name(&self) -> String {
        "LocalLibrarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
        self.local_library_search(query)
    }    

    fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
        self.local_library.list(of_type, related_to)
    }

    fn fetch(&self, entity: &Entities) -> Option<Entities> {
        self.local_library.fetch(&entity)
    }

    fn image(&self, entity: &Entities) -> Option<DynamicImage> {
        self.local_library.image(entity)
    }

    fn stream(&self, entity: &Entities) -> Option<Box<dyn Iterator<Item = u8>>> {
        self.local_library.stream(entity)
    }    
}

