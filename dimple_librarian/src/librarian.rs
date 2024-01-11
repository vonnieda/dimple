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
        // TODO remove dupes
        log::info!("{}: {}", "Search".cyan(), query.blue());
        let merged: Vec<LibraryEntity> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| {
                log::info!("  {} {}", "✔".bright_green(), lib.name().green());
                lib.search(query)
            })
            .collect();
        Box::new(merged.into_iter())
    }    

    fn artists(&self) -> Box<dyn Iterator<Item = Artist>> {
        self.local_library.artists()
    }

    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        let fetch_and_log = |lib: &dyn Library, entity: &LibraryEntity| {
            lib.fetch(entity)
                .map(|ent| { 
                    log::info!("  {} {}", "✔".bright_green(), lib.name().bright_green());
                    ent
                })
                .or_else(|| {
                    log::info!("  {} {}", "✗".bright_red(), lib.name().bright_red());
                    None
                })
        };

        let store_and_log = |entity: LibraryEntity| {
            match &entity {
                LibraryEntity::Artist(artist) => self.local_library.set_artist(artist),
                LibraryEntity::Genre(_) => todo!(),
                LibraryEntity::Release(_) => todo!(),
                LibraryEntity::Track(_) => todo!(),
            };
            entity
        };

        log::info!("{} {} ({})", "Fetch".green(), entity.name().blue(), entity.mbid().yellow());
        fetch_and_log(&self.local_library, entity)
            .or_else(|| {
                self.libraries.read().ok()?.iter()
                    .find_map(|lib| fetch_and_log(lib.as_ref(), entity))
                    .map(store_and_log)
            }
        )
    }
    
    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        let fetch_and_log = |lib: &dyn Library, entity: &LibraryEntity| {
            let result = lib.image(entity);
            if result.is_some() {
                log::info!("  {} {}", "✔".bright_green(), lib.name().green());
            }
            else {
                log::info!("  {} {}", "✗".bright_red(), lib.name().bright_red());
            }
            result
        };

        log::info!("{} {} ({})", "Image".magenta(), entity.name().blue(), entity.mbid().yellow());
        fetch_and_log(&self.local_library, entity)
            .or_else(|| {
                self.libraries.read().ok()?.iter()
                    .find_map(|lib| fetch_and_log(lib.as_ref(), entity))
                    .map(|dyn_image| {
                        self.local_library.set_image(entity, &dyn_image);
                        dyn_image
                    })
            }
        )
    }
}

// std::iter::once(&self.local_library as &dyn Library)
// .chain(self.libraries.read().ok()?.iter().map(|l| &**l as &dyn Library))
// .find_map(|lib| {
//     let result = lib.fetch(entity);
//     if result.is_some() {
//         log::info!("  {} {}", "✔".bright_green(), lib.name().green());
//     }
//     result
// })
// .map(|ent| {
//     match ent.clone() {
//         LibraryEntity::Artist(artist) => self.local_library.set_artist(&artist),
//         LibraryEntity::Genre(_) => todo!(),
//         LibraryEntity::Release(_) => todo!(),
//         LibraryEntity::Track(_) => todo!(),
//     };
//     ent
// })
