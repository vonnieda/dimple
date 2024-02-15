use std::{sync::{RwLock, Mutex}, collections::HashSet};

use dimple_core::{collection::Collection, model::{Artist, Entity, Recording, RecordingSource, Release, ReleaseGroup}};
use dimple_sled_library::sled_library::SledLibrary;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use image::DynamicImage;
use rayon::prelude::*;
use dimple_core::model::Entities;

pub struct Librarian {
    local_library: SledLibrary,
    libraries: RwLock<Vec<Box<dyn Collection>>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        Self { 
            local_library: SledLibrary::new(path),
            libraries: Default::default(), 
        }
    }

    pub fn add_library(&self, library: Box<dyn Collection>) {
        self.libraries.write().unwrap().push(library);
    }

    /// Generate some kind of cool artwork for the entity to be used as a
    /// default. Being part of Librarian, it can use data from the library
    /// to create the image.
    pub fn generate_masterpiece(&self, _entity: &Entities, width: u32, 
        height: u32) -> DynamicImage {

        // https://stackoverflow.com/questions/76741218/in-slint-how-do-i-render-a-self-drawn-image
        // http://ia802908.us.archive.org/35/items/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c-25647975198.jpg

        DynamicImage::new_rgb8(width, height)
    }

    /// Get or create a thumbnail image at the given size for the entity.
    /// If no image can be loaded from the library one is generated. Results
    /// either from the library or generated are cached for future calls.
    pub fn thumbnail(&self, entity: &Entities, width: u32, height: u32) -> DynamicImage {
        // let cached = self.local_library.images.get(&entity.key(), width, height);
        // if let Some(dyn_image) = cached {
        //     return dyn_image;
        // }
        // else if let Some(dyn_image) = self.image(entity) {
        //     self.local_library.set_image(entity, &dyn_image);
        //     return self.local_library.images.get(&entity.key(), width, height).unwrap();
        // }
        // let generated = &self.generate_masterpiece(entity, width, height);
        // self.local_library.set_image(entity, generated);
        // self.local_library.images.get(&entity.key(), width, height).unwrap()

        DynamicImage::new_rgb8(32, 32)
    }

    /// Find a matching model in the local store merge the value, and store it. 
    /// If no match is found a new model is stored and returned.
    pub fn merge(&self, model: &Entities, related_to: Option<&Entities>) -> Entities {
        // TODO this needs to be atomic. When using par_iter it blows up cause
        // we're saving out of date info.
        let matched = self.find_match(model);
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

    // 1. Attempt to find by key
    // 2. Attempt to find by source_id
    // 3. Attempt to find by known_id
    // 4. Attempt to find by fuzzy?
    // 5. Give up.
    fn find_match(&self, model: &Entities) -> Option<Entities> {
        // Find by key
        if let Some(model) = self.local_library.get(model) {
            return Some(model)
        }

        // Find by matching source_id
        if let Some(model) = self.local_library.list(model, None).find(|m| {
            let l = model;
            let r = m;
            !l.source_ids().is_disjoint(&r.source_ids())
        }) {
            return Some(model)
        }

        // Find by matching known_id
        if let Some(model) = self.local_library.list(model, None).find(|m| {
            let l = model;
            let r = m;
            !l.known_ids().is_disjoint(&r.known_ids())
        }) {
            return Some(model)
        }

        // Find by fuzzy 
        // TODO score, sort
        // TODO I think this becomes LocalLibrary.search or maybe a utility
        // for an iterator.
        if let Some(model) = self.local_library.list(model, None).find(|m| {
            let l = model;
            let r = m;
            let matcher = SkimMatcherV2::default();
            let pattern = format!("{}:{}", 
                l.name().unwrap_or_default(),
                l.disambiguation().unwrap_or_default());
            let choice = format!("{}:{}", 
                r.name().unwrap_or_default(),
                r.disambiguation().unwrap_or_default());
            let score = matcher.fuzzy_match(&choice, &pattern);
            // log::info!("{}, {} -> {}", choice, pattern, score.unwrap_or(0));
            score.is_some()
        }) {
            return Some(model)
        }

        // Give up
        None
    }

    fn local_library_search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
        let matcher = SkimMatcherV2::default();
        let pattern = query.to_string();
        let iter = Artist::list(&self.local_library)
            .filter(move |a| matcher.fuzzy_match(&a.name.clone().unwrap_or_default(), &pattern).is_some())
            .map(Entities::Artist);
        Box::new(iter)
    }
}

impl Collection for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
        self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.search(query))
            .for_each(|m| {
                let _ = self.merge(&m, None);
            });
        // self.local_library.search(query)
        self.local_library_search(query)
    }    

    fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
        // TODO parallel version is much faster but sled.merge is not atomic
        // / thread safe, so it fucks up. see note on that function.
        // self.libraries.read().unwrap().par_iter()
        //     .flat_map(|lib| lib.list(of_type, related_to).collect::<Vec<_>>())
        //     .for_each(|m| {
        //         let _ = self.merge(&m, related_to);
        //     });

        self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.list(of_type, related_to))
            .for_each(|m| {
                let _ = self.merge(&m, related_to);
            });

        self.local_library.list(of_type, related_to)
    }

    fn fetch(&self, entity: &Entities) -> Option<Entities> {
        self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.list(entity, None))
            .for_each(|m| {
                let _ = self.merge(&m, None);
            });
        self.local_library.fetch(entity)
    }

    /// If there is an image stored in the local library for the entity return
    /// it, otherwise search the attached libraries for one. If one is found,
    /// cache it in the local library and return it. Future requests will be
    /// served from the cache.
    fn image(&self, entity: &Entities) -> Option<DynamicImage> {
        todo!()
    }
}

trait Merge<T> {
    // TODO should probably be references.
    fn merge(a: T, b: T) -> T;
}

impl Merge<Entities> for Entities {
    fn merge(left: Entities, right: Entities) -> Self {
        match (left, right) {
            (Entities::Artist(left), Entities::Artist(right)) => {
                Artist::merge(left, right).entity()
            },
            (Entities::ReleaseGroup(left), Entities::ReleaseGroup(right)) => {
                ReleaseGroup::merge(left, right).entity()
            },
            (Entities::Release(left), Entities::Release(right)) => {
                Release::merge(left, right).entity()
            },
            (Entities::Recording(left), Entities::Recording(right)) => {
                Recording::merge(left, right).entity()
            },
            (Entities::RecordingSource(left), Entities::RecordingSource(right)) => {
                RecordingSource::merge(left, right).entity()
            },
            _ => todo!()
        }
    }
}

impl Merge<Self> for Artist {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            name: a.name.or(b.name),
            summary: a.summary.or(b.summary),
        }
    }
}

impl Merge<Self> for ReleaseGroup {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            title: a.title.or(b.title),
            summary: a.summary.or(b.summary),

            first_release_date: a.first_release_date.or(b.first_release_date),
            primary_type: a.primary_type.or(b.primary_type),
        }
    }
}

impl Merge<Self> for Release {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            title: a.title.or(b.title),
            summary: a.summary.or(b.summary),


            barcode: a.barcode.or(b.barcode),
            country: a.country.or(b.country),
            date: a.date.or(b.date),
            packaging: a.packaging.or(b.packaging),
            status: a.status.or(b.status),
        }
    }
}

impl Merge<Self> for Recording {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            title: a.title.or(b.title),
            summary: a.summary.or(b.summary),

            annotation: a.annotation.or(b.annotation),
            isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
            length: a.length.or(b.length)
        }
    }
}

impl Merge<Self> for RecordingSource {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            // disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            // links: a.links.union(&b.links).cloned().collect(),
            // title: a.title.or(b.title),
            // summary: a.summary.or(b.summary),

            // annotation: a.annotation.or(b.annotation),
            // isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
            // length: a.length.or(b.length)
        }
    }
}


    // /// Get or create a thumbnail image at the given size for the entity.
    // /// If no image can be loaded from the library one is generated. Results
    // /// either from the library or generated are cached for future calls.
    // pub fn thumbnail(&self, entity: &Model, width: u32, height: u32) -> DynamicImage {
    //     let cached = self.local_library.images.get(&entity.key(), width, height);
    //     if let Some(dyn_image) = cached {
    //         return dyn_image;
    //     }
    //     else if let Some(dyn_image) = self.image(entity) {
    //         self.local_library.set_image(entity, &dyn_image);
    //         return self.local_library.images.get(&entity.key(), width, height).unwrap();
    //     }
    //     let generated = &self.generate_masterpiece(entity, width, height);
    //     self.local_library.set_image(entity, generated);
    //     self.local_library.images.get(&entity.key(), width, height).unwrap()
    // }

    // fn fetch_with_force(&self, entity: &Model, force: bool) -> Option<Model> {
    //     if !force {
    //         let local_result = self.local_library.fetch(entity);
    //         if local_result.is_some() {
    //             return local_result;
    //         }
    //     } 

    //     // Run the fetch on all of the libraries, keeping track of the ones
    //     // that return a good result. 
    //     let skip_libs: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    //     let first_result: Model = self.libraries.read().ok()?.par_iter()
    //         .filter_map(|lib| {
    //             let result = lib.fetch(entity);
    //             if result.is_some() {
    //                 skip_libs.lock().unwrap().insert(lib.name());
    //             }
    //             result
    //         })
    //         .reduce(|| entity.clone(), Model::merge);
        
    //     // Run the fetch on the remaining libraries that did not return a result
    //     // the first time. This allows libraries that need metadata from
    //     // Musicbrainz to skip the first fetch and run on this one.
    //     let second_result: Model = self.libraries.read().ok()?.par_iter()
    //         .filter(|f| !skip_libs.lock().unwrap().contains(&f.name()))
    //         .filter_map(|lib| lib.fetch(&first_result))
    //         .reduce(|| entity.clone(), Model::merge);

    //     // Merge the results together, store it for later access, and return
    //     let result = Model::merge(first_result, second_result);
    //     self.local_library.merge(&result, None);

    //     Some(result)
    // }

