use std::{collections::HashSet, sync::{Mutex, RwLock}};

use dimple_core::{collection::Collection, model::{Artist, Recording, RecordingSource, Release, ReleaseGroup}};
use dimple_sled_library::sled_library::SledLibrary;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use image::DynamicImage;
use rayon::prelude::*;
use dimple_core::model::Entities;

use crate::{art_gen, matching};

pub struct Librarian {
    local_library: SledLibrary,
    libraries: RwLock<Vec<Box<dyn Collection>>>,
    access_mode: Mutex<AccessMode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessMode {
    Online,
    Offline,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        Self { 
            local_library: SledLibrary::new(path),
            libraries: Default::default(),
            access_mode: Mutex::new(AccessMode::Online),
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
    /// TODO need to think about how this merges after an offline session, and
    /// I think that moves it more towards a reusable component that happens
    /// in the foreground when needed but mostly in the background.
    /// TODO Okay, I think this is going to start storing every entity, matching
    /// maybe only on source id, and then what the UI will get will be merged in
    /// real time? Or also merged at the same time, but primarily gonna start
    /// storing and being able to reference all the source objects.
    /// Oh wow, and then I could just have the queries run on the dimple source
    /// ids. Or even librarian.
    pub fn merge(&self, model: &Entities, related_to: Option<&Entities>) -> Entities {
        // TODO this needs to be atomic. When using par_iter it blows up cause
        // we're saving out of date info.
        let matched = matching::find_match(self, &self.local_library, model);
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

    fn local_library_search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
        // const MAX_RESULTS_PER_TYPE: usize = 10;
        // TODO sort by score
        // TODO other entities
        let pattern = query.to_string();
        let matcher = SkimMatcherV2::default();
        let artists = Artist::list(&self.local_library)
            .filter(move |a| matcher.fuzzy_match(&a.name.clone().unwrap_or_default(), &pattern).is_some())
            .map(Entities::Artist);
            // .take(MAX_RESULTS_PER_TYPE);
        let pattern = query.to_string();
        let matcher = SkimMatcherV2::default();
        let release_groups = ReleaseGroup::list(&self.local_library)
            .filter(move |a| matcher.fuzzy_match(&a.title.clone().unwrap_or_default(), &pattern).is_some())
            .map(Entities::ReleaseGroup);
            // .take(MAX_RESULTS_PER_TYPE);
        Box::new(artists.chain(release_groups))
    }
}

impl Collection for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    /// Search the libraries, merge the results to local, and then search
    /// local and return the results. This merge step ensures that the objects
    /// returned are the sum of the sources.
    fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
        if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
            self.libraries.read().unwrap().iter()
                .flat_map(|lib| lib.search(query))
                .for_each(|m| {
                    let _ = self.merge(&m, None);
                });
        }
        self.local_library_search(query)
    }    

    /// List the libraries, merge the results to local, and then list
    /// local and return the results. This merge step ensures that the objects
    /// returned are the sum of the sources.
    fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
        // TODO parallel version is much faster but self.merge is not atomic
        // But note, the real problem is the "list" in the match function.
        if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
            self.libraries.read().unwrap().iter()
                .flat_map(|lib| lib.list(of_type, related_to))
                .for_each(|m| {
                    let _ = self.merge(&m, related_to);
                });
        }
        self.local_library.list(of_type, related_to)
    }

    /// First, merge the request entity as a side effect. Search the libraries, 
    /// merge the results to local, and then search local and return the 
    /// results. This merge step ensures that the objects returned are the 
    /// sum of the sources.
    fn fetch(&self, entity: &Entities) -> Option<Entities> {
        // Merging the entity before searching ensures we have things like
        // additional ids for querying. I'm not totally sure if this should
        // merge (to disk) or if it should query and merge in memory for
        // the query only. Since we're gonna save it at the end anyway I don't
        // see any harm in saving it here too. This could be replaced with
        // find_match and merge in memory though.

        // TODO I don't think this logic is totally sound. At the end we fetch
        // using this entity, but what if we've merged something? I think we
        // need to replace the query with the _ at the end of the sub-block.
        // This may possibly explain the weird bug where Spidergawd shows up in
        // Opeth

        let entity = self.merge(entity, None);

        if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
            // Run the fetch on all of the libraries, keeping track of the ones
            // that return a result. 
            let skip_libs: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
            let first_result: Entities = self.libraries.read().ok()?.par_iter()
                .filter_map(|lib| {
                    let result = lib.fetch(&entity);
                    if result.is_some() {
                        skip_libs.lock().unwrap().insert(lib.name());
                    }
                    result
                })
                .reduce(|| entity.clone(), Entities::merge);
            
            // Run the fetch on the remaining libraries that did not return a result
            // the first time. This allows libraries that need metadata from
            // for instance, Musicbrainz to skip the first fetch and run on
            // this one instead.
            let second_result: Entities = self.libraries.read().ok()?.par_iter()
                .filter(|f| !skip_libs.lock().unwrap().contains(&f.name()))
                .filter_map(|lib| lib.fetch(&first_result))
                .reduce(|| entity.clone(), Entities::merge);

            // Merge the results together and store it for later access
            let result = Entities::merge(first_result, second_result);
            let _ = self.merge(&result, None);
        }

        // Return the results from the local library
        self.local_library.fetch(&entity)
    }

    /// If there is an image stored in the local library for the entity return
    /// it, otherwise search the attached libraries for one. If one is found,
    /// cache it in the local library and return it. Future requests will be
    /// served from the cache.
    /// TODO I think this goes away and becomes fetch(Dimage), maybe Dimage
    /// so it's not constantly overlapping with Image.
    fn image(&self, entity: &Entities) -> Option<DynamicImage> {
        let image = self.local_library.image(entity);
        if image.is_some() {
            return image;
        }

        if self.access_mode.lock().unwrap().clone() == AccessMode::Online {
            if let Some(dyn_image) = self.libraries.read().unwrap().iter().find_map(|lib| lib.image(entity)) {
                self.local_library.set_image(entity, &dyn_image);
                return Some(dyn_image)
            }
        }
        None
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
            country: a.country.or(b.country),
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
            format: a.format.or(b.format),
            extension: a.extension.or(b.extension),
            // links: a.links.union(&b.links).cloned().collect(),
            // title: a.title.or(b.title),
            // summary: a.summary.or(b.summary),

            // annotation: a.annotation.or(b.annotation),
            // isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
            // length: a.length.or(b.length)
        }
    }
}
