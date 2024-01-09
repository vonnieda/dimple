use std::sync::RwLock;

use dimple_core::{library::{Library, LibraryEntity}, model::Artist};
use dimple_sled_library::sled_library::SledLibrary;
use image::DynamicImage;

pub struct Librarian {
    local_library: SledLibrary,
    libraries: RwLock<Vec<Box<dyn Library>>>,
}

impl Default for Librarian {
    fn default() -> Self {
        Self { 
            local_library: SledLibrary::new("local_library", "local_library"),
            libraries: Default::default(), 
        }
    }    
}

impl Librarian {
    pub fn add_library(&self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(library);
    }

    fn merge_artist(src: &Artist, dest: &Artist) -> Artist {
        let mut dest = dest.clone();
        if dest.mbid.is_none() {
            dest.mbid = src.mbid.clone();
        }
        if dest.name.is_empty() {
            dest.name = src.name.clone();
        }
        dest
    }

    fn resolve(&self, e: &LibraryEntity) -> LibraryEntity {
        match e {
            LibraryEntity::Artist(a_in) => {
                let artist = self.local_library.get_artist_by_id(&a_in.id)
                    .or_else(|| self.local_library.get_artist_by_mbid(a_in.mbid.clone()))
                    .or_else(|| Some(Artist::default()))
                    .map(|a| Self::merge_artist(a_in, &a))
                    .map(|a| {
                        self.local_library.set_artist(&a);
                        a
                    })
                    .unwrap();
                LibraryEntity::Artist(artist)
            }
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(_) => todo!(),
            LibraryEntity::Track(_) => todo!(),
        }
    }
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = dimple_core::library::LibraryEntity>> {
        let merged: Vec<LibraryEntity> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.search(query))
            .map(|e| self.resolve(&e))
            .collect();
        Box::new(merged.into_iter())
    }    

    fn artists(&self) -> Box<dyn Iterator<Item = Artist>> {
        self.local_library.artists()
    }

    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        if let Some(dyn_image) = self.local_library.image(entity) {
            return Some(dyn_image);
        }
        for lib in self.libraries.read().unwrap().iter() {
            log::debug!("Searching {} for  {:?}", lib.name(), entity);
            if let Some(dyn_image) = lib.image(entity) {
                self.local_library.set_image(entity, &dyn_image);
                return Some(dyn_image);
            }
        }
        log::warn!("no image found for {:?}, setting default", entity);
        let dyn_image = DynamicImage::new_rgba8(500, 500);
        self.local_library.set_image(entity, &dyn_image);
        Some(dyn_image)
    }
}

