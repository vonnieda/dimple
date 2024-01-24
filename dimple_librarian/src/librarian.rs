use std::{sync::{RwLock, Mutex}, collections::HashSet, any::Any};

use colored::Colorize;
use dimple_core::{library::{Library, LibraryEntity}, model::{DimpleArtist, DimpleReleaseGroup, DimpleRelease, DimpleRecording}};
use dimple_sled_library::sled_library::SledLibrary;
use image::DynamicImage;
use rayon::prelude::*;

// TODO need a favicon service


pub struct Librarian {
    /// TODO It feels like it's about time to retire SledLibrary, and move the
    /// Sled stuff here. I'm going to need additional trees for configs,
    /// user images, and other stuff. I had considered just having SledLibrary
    /// take a Tree, and then I could control the root here, but then SledLibrary
    /// is basically doing nothing but serializing which is easy enough to do here.
    /// Then I don't have to be retricted to the Library operations for the stuff
    /// that happens here.
    local_library: SledLibrary,
    libraries: RwLock<Vec<Box<dyn Library>>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        Self { 
            local_library: SledLibrary::new(path),
            libraries: Default::default(), 
        }
    }

    pub fn add_library(&self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(library);
    }

    /// Generate some kind of cool artwork for the entity to be used as a
    /// default. Being part of Librarian, it can use data from the library
    /// to create the image.
    pub fn generate_masterpiece(&self, entity: &LibraryEntity, width: u32, 
        height: u32) -> DynamicImage {


        // https://stackoverflow.com/questions/76741218/in-slint-how-do-i-render-a-self-drawn-image
        // http://ia802908.us.archive.org/35/items/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c-25647975198.jpg


        DynamicImage::new_rgb8(width, height)
    }

    /// Get or create a thumbmail image at the given size for the entity.
    /// If no image can be loaded from the library one is generated. Results
    /// either from the library or generated are cached for future calls.
    pub fn thumbnail(&self, entity: &LibraryEntity, width: u32, height: u32) -> DynamicImage {
        let cached = self.local_library.images.get(&entity.id(), width, height);
        if let Some(dyn_image) = cached {
            return dyn_image;
        }
        else if let Some(dyn_image) = self.image(entity) {
            self.local_library.set_image(entity, &dyn_image);
            return self.local_library.images.get(&entity.id(), width, height).unwrap();
        }
        let generated = &self.generate_masterpiece(entity, width, height);
        self.local_library.set_image(entity, generated);
        self.local_library.images.get(&entity.id(), width, height).unwrap()
    }

    fn fetch_with_force(&self, entity: &LibraryEntity, force: bool) -> Option<LibraryEntity> {
        if !force {
            let local_result = self.local_library.fetch(entity);
            if local_result.is_some() {
                return local_result;
            }
        } 

        // TODO I think ultimately to solve the don't query twice problem
        // I need to change the interface to Result, so that I can know
        // here if there was an error and it's worth trying again, or if
        // it worked but there was no result.
        // So like something like WikiData could return Err if there was no
        // wikidata link. 
        let skip_libs: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
        let first_result: LibraryEntity = self.libraries.read().ok()?.par_iter()
            .filter_map(|lib| {
                let result = lib.fetch(entity);
                if result.is_some() {
                    skip_libs.lock().unwrap().insert(lib.name());
                }
                result
            })
            .reduce(|| entity.clone(), LibraryEntity::merge);
        
        let second_result: LibraryEntity = self.libraries.read().ok()?.par_iter()
            .filter(|f| !skip_libs.lock().unwrap().contains(&f.name()))
            .filter_map(|lib| lib.fetch(&first_result))
            .reduce(|| entity.clone(), LibraryEntity::merge);

        let result = LibraryEntity::merge(first_result, second_result);
        self.local_library.store(&result);

        Some(result)
    }
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = dimple_core::library::LibraryEntity>> {
        // TODO include local
        // TODO remove dupes
        log::debug!("{}: {}", "Search".cyan(), query.blue());
        let merged: Vec<LibraryEntity> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| {
                log::debug!("  {} {}", "✔".bright_green(), lib.name().green());
                lib.search(query)
            })
            .collect();
        Box::new(merged.into_iter())
    }    

    fn list(&self, entity: &LibraryEntity) -> Box<dyn Iterator<Item = LibraryEntity>> {
        self.local_library.list(entity)
    }

    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        self.fetch_with_force(entity, false)
    }

    /// If there is an image stored in the local library for the entity return
    /// it, otherwise search the attached libraries for one. If one is found,
    /// cache it in the local library and return it. Future requests will be
    /// served from the cache.
    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        let image = self.local_library.image(entity);
        if image.is_some() {
            return image;
        }

        self.libraries.read().ok()?.par_iter()
            .find_map_first(|lib| lib.image(entity))
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

impl Merge<LibraryEntity> for LibraryEntity {
    fn merge(left: LibraryEntity, right: LibraryEntity) -> Self {
        match left {
            LibraryEntity::Artist(left) => match right {
                LibraryEntity::Artist(right) => LibraryEntity::Artist(DimpleArtist::merge(left, right)),
                _ => panic!("no")
            },
            LibraryEntity::ReleaseGroup(left) => match right {
                LibraryEntity::ReleaseGroup(right) => LibraryEntity::ReleaseGroup(DimpleReleaseGroup::merge(left, right)),
                _ => panic!("no")
            },
            LibraryEntity::Release(left) => match right {
                LibraryEntity::Release(right) => LibraryEntity::Release(DimpleRelease::merge(left, right)),
                _ => panic!("no")
            },
            LibraryEntity::Recording(left) => match right {
                LibraryEntity::Recording(right) => LibraryEntity::Recording(DimpleRecording::merge(left, right)),
                _ => panic!("no")
            },
            _ => panic!("no")
        }
    }
}

impl Merge<Self> for DimpleArtist {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.disambiguation = longer(base.disambiguation, b.disambiguation);
        base.genres = merge_vec(base.genres, b.genres);
        base.id = longer(base.id, b.id);
        base.name = longer(base.name, b.name);
        base.relations = merge_vec(base.relations, b.relations);
        base.release_groups = merge_vec(base.release_groups, b.release_groups);
        base.summary = longer(base.summary, b.summary);
        base
    }
}

impl Merge<Self> for DimpleReleaseGroup {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.disambiguation = longer(base.disambiguation, b.disambiguation);
        base.first_release_date = longer(base.first_release_date, b.first_release_date);
        base.genres = merge_vec(base.genres, b.genres);
        base.id = longer(base.id, b.id);
        base.primary_type = longer(base.primary_type, b.primary_type);
        base.relations = merge_vec(base.relations, b.relations);
        base.releases = merge_vec(base.releases, b.releases);
        base.summary = longer(base.summary, b.summary);
        base.title = longer(base.title, b.title);
        base.artists = merge_vec(base.artists, b.artists);
        base
    }
}

impl Merge<Self> for DimpleRelease {
    fn merge(base: Self, b: Self) -> Self {
        let mut base = base.clone();
        base.artists = merge_vec(base.artists, b.artists);
        base.barcode = longer(base.barcode, b.barcode);
        base.country = longer(base.country, b.country);
        base.date = longer(base.date, b.date);
        base.disambiguation = longer(base.disambiguation, b.disambiguation);
        base.genres = merge_vec(base.genres, b.genres);
        base.media = merge_vec(base.media, b.media);
        base.id = longer(base.id, b.id);
        base.relations = merge_vec(base.relations, b.relations);
        base.status = longer(base.status, b.status);
        base.summary = longer(base.summary, b.summary);
        base.title = longer(base.title, b.title);
        base
    }
}

impl Merge<Self> for DimpleRecording {
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
        base.id = longer(base.id, b.id);
        // base.length = 
        // base.relations = merge_vec(base.relations, b.relations);
        // base.status = longer(base.status, b.status);
        base.summary = longer(base.summary, b.summary);
        base.title = longer(base.title, b.title);
        base
    }
}
