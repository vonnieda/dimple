use std::{sync::{RwLock, Mutex}, collections::HashSet};

use dimple_core::{collection::{Collection}, model::{Artist, ReleaseGroup, Release, Recording}};
use dimple_sled_library::sled_library::SledLibrary;
use image::DynamicImage;
use rayon::prelude::*;
use dimple_core::model::Model;

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
    pub fn generate_masterpiece(&self, entity: &Model, width: u32, 
        height: u32) -> DynamicImage {


        // https://stackoverflow.com/questions/76741218/in-slint-how-do-i-render-a-self-drawn-image
        // http://ia802908.us.archive.org/35/items/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c-25647975198.jpg


        DynamicImage::new_rgb8(width, height)
    }

    /// Get or create a thumbmail image at the given size for the entity.
    /// If no image can be loaded from the library one is generated. Results
    /// either from the library or generated are cached for future calls.
    pub fn thumbnail(&self, entity: &Model, width: u32, height: u32) -> DynamicImage {
        let cached = self.local_library.images.get(&entity.key(), width, height);
        if let Some(dyn_image) = cached {
            return dyn_image;
        }
        else if let Some(dyn_image) = self.image(entity) {
            self.local_library.set_image(entity, &dyn_image);
            return self.local_library.images.get(&entity.key(), width, height).unwrap();
        }
        let generated = &self.generate_masterpiece(entity, width, height);
        self.local_library.set_image(entity, generated);
        self.local_library.images.get(&entity.key(), width, height).unwrap()
    }

    fn fetch_with_force(&self, entity: &Model, force: bool) -> Option<Model> {
        if !force {
            let local_result = self.local_library.fetch(entity);
            if local_result.is_some() {
                return local_result;
            }
        } 

        // Run the fetch on all of the libraries, keeping track of the ones
        // that return a good result. 
        let skip_libs: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
        let first_result: Model = self.libraries.read().ok()?.par_iter()
            .filter_map(|lib| {
                let result = lib.fetch(entity);
                if result.is_some() {
                    skip_libs.lock().unwrap().insert(lib.name());
                }
                result
            })
            .reduce(|| entity.clone(), Model::merge);
        
        // Run the fetch on the remaining libraries that did not return a result
        // the first time. This allows libraries that need metadata from
        // Musicbrainz to skip the first fetch and run on this one.
        let second_result: Model = self.libraries.read().ok()?.par_iter()
            .filter(|f| !skip_libs.lock().unwrap().contains(&f.name()))
            .filter_map(|lib| lib.fetch(&first_result))
            .reduce(|| entity.clone(), Model::merge);

        // Merge the results together, store it for later access, and return
        let result = Model::merge(first_result, second_result);
        self.local_library.store(&result);

        Some(result)
    }
}

impl Collection for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = Model>> {
        // TODO include local
        // TODO remove dupes
        let merged: Vec<Model> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| {
                lib.search(query)
            })
            .collect();
        Box::new(merged.into_iter())
    }    

    fn list(&self, of_type: &Model, related_to: Option<&Model>) -> Box<dyn Iterator<Item = Model>> {
        let local_results: Vec<_> = self.local_library.list(of_type, related_to).collect();
        let lib_results: Vec<_> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.list(of_type, related_to))
            .collect();

        let mut merged = vec![];
        merged.extend_from_slice(&local_results);
        merged.extend_from_slice(&lib_results);
        Box::new(merged.into_iter())
    }

    fn fetch(&self, entity: &Model) -> Option<Model> {
        self.fetch_with_force(entity, false)
    }

    /// If there is an image stored in the local library for the entity return
    /// it, otherwise search the attached libraries for one. If one is found,
    /// cache it in the local library and return it. Future requests will be
    /// served from the cache.
    fn image(&self, entity: &Model) -> Option<DynamicImage> {
        let image = self.local_library.image(entity);
        if image.is_some() {
            return image;
        }

        self.libraries.read().ok()?.iter()
            .find_map(|lib| lib.image(entity))
            .map(|dyn_image| {
                self.local_library.set_image(entity, &dyn_image);
                dyn_image
            })
    }
}

trait Merge<T> {
    fn merge(a: T, b: T) -> T;
}

fn longer(a: String, b: String) -> String {
    if a.len() > b.len() { a }
    else { b }
}

fn merge_vec<T>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    if a.len() > b.len() { a } else { b }
}

impl Merge<Model> for Model {
    fn merge(left: Model, right: Model) -> Self {
        match (left, right) {
            (Model::Artist(left), Model::Artist(right)) => {
                Artist::merge(left, right).entity()
            },
            _ => todo!()
        }
    }
}

impl Merge<Self> for Artist {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.disambiguation = base.disambiguation.or(b.disambiguation);
        // base.genres = merge_vec(base.genres, b.genres);
        base.key = longer(base.key, b.key);
        base.name = base.name.or(b.name);
        // base.relations = merge_vec(base.relations, b.relations);
        // base.release_groups = merge_vec(base.release_groups, b.release_groups);
        base.summary = base.summary.or(b.summary);
        base
    }
}

impl Merge<Self> for ReleaseGroup {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.disambiguation = longer(base.disambiguation, b.disambiguation);
        base.first_release_date = longer(base.first_release_date, b.first_release_date);
        base.genres = merge_vec(base.genres, b.genres);
        base.key = longer(base.key, b.key);
        base.primary_type = longer(base.primary_type, b.primary_type);
        base.relations = merge_vec(base.relations, b.relations);
        base.releases = merge_vec(base.releases, b.releases);
        base.summary = longer(base.summary, b.summary);
        base.title = longer(base.title, b.title);
        base.artists = merge_vec(base.artists, b.artists);
        base
    }
}

impl Merge<Self> for Release {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.artists = merge_vec(base.artists, b.artists);
        base.barcode = longer(base.barcode, b.barcode);
        base.country = longer(base.country, b.country);
        base.date = longer(base.date, b.date);
        base.disambiguation = longer(base.disambiguation, b.disambiguation);
        base.genres = merge_vec(base.genres, b.genres);
        base.media = merge_vec(base.media, b.media);
        base.key = longer(base.key, b.key);
        base.relations = merge_vec(base.relations, b.relations);
        base.status = longer(base.status, b.status);
        base.summary = longer(base.summary, b.summary);
        base.title = longer(base.title, b.title);
        base
    }
}

impl Merge<Self> for Recording {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.annotation = longer(base.annotation, b.annotation);
        base.artist_credits = merge_vec(base.artist_credits, b.artist_credits);
        // base.asin = longer(base.asin, b.asin);
        // base.country = longer(base.country, b.country);
        // base.date = longer(base.date, b.date);
        // base.barcode = longer(base.barcode, b.barcode);
        base.disambiguation = longer(base.disambiguation, b.disambiguation);
        // base.genres = merge_vec(base.genres, b.genres);
        // base.media = merge_vec(base.media, b.media);
        base.key = longer(base.key, b.key);
        // base.length = 
        // base.relations = merge_vec(base.relations, b.relations);
        // base.status = longer(base.status, b.status);
        base.summary = longer(base.summary, b.summary);
        base.title = longer(base.title, b.title);
        base
    }
}
