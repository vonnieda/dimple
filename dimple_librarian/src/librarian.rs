use std::sync::RwLock;

use colored::Colorize;
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
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = dimple_core::library::LibraryEntity>> {
        // TODO include local
        let merged: Vec<LibraryEntity> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.search(query))
            // TODO remove dupes
            .collect();
        Box::new(merged.into_iter())
    }    

    fn artists(&self) -> Box<dyn Iterator<Item = Artist>> {
        self.local_library.artists()
    }

    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        self.local_library.fetch(entity).or_else(|| {
            self.libraries.read().ok()?.iter()            
                .find_map(|lib| lib.fetch(entity))
                .map(|e| {
                    match e.clone() {
                        LibraryEntity::Artist(a) => self.local_library.set_artist(&a),
                        LibraryEntity::Genre(_) => todo!(),
                        LibraryEntity::Release(_) => todo!(),
                        LibraryEntity::Track(_) => todo!(),
                    };
                    e
                })
        })
    }
    
    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        log::info!("{} {}", "Find images for".blue(), entity.name().blue());
        if let Some(dyn_image) = self.local_library.image(entity) {
            log::info!("  {} {}", "✔".green(), self.local_library.name().green());
            return Some(dyn_image);
        }
        log::info!("  {} {}", "✗".red(), self.local_library.name().red());
        for lib in self.libraries.read().unwrap().iter() {
            if let Some(dyn_image) = lib.image(entity) {
                log::info!("  {} {}", "✔".green(), lib.name().green());
                self.local_library.set_image(entity, &dyn_image);
                return Some(dyn_image);
            }
            else {
                log::info!("  {} {}", "✗".red(), lib.name().red());
            }
        }
        None
        // log::debug!("no image found for {} ({}), setting default", entity.name(), entity.mbid());
        // let dyn_image = DynamicImage::new_rgba8(500, 500);
        // self.local_library.set_image(entity, &dyn_image);
        // Some(dyn_image)
    }
}

