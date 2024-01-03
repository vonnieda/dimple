use std::sync::RwLock;

use dimple_core::{library::{Library, LibraryEntity}, model::Artist};
use dimple_sled_library::sled_library::SledLibrary;
use ulid::Ulid;

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

    pub fn create_id(&self) -> String {
        Ulid::new().to_string()
    }

    // Update or create the entity in the local library, returning the local
    // entity.
    fn resolve(&self, e: &LibraryEntity) -> LibraryEntity {
        log::info!("resolve {:?}", e);
        match e {
            LibraryEntity::Artist(a_in) => {
                self.local_library
                    // Get the existing entity by id
                    .get_artist(&a_in.id)

                    // Or by mbid
                    .or_else(|| self.local_library.get_artist_by_mbid(a_in.mbid.clone()))

                    // Or create a new one with a new id
                    .or_else(|| {
                        let a = Artist {
                            id: Ulid::new().to_string(),
                            ..Default::default()
                        };
                        // log::info!("Created new artist {}", a.id);
                        Some(a)
                    })

                    // Update it with any missing properties
                    .map(|mut a| {
                        if a.mbid.is_none() {
                            a.mbid = a_in.mbid.clone();
                        }
                        if a.name.is_empty() {
                            a.name = a_in.name.clone();
                        }
                        a
                    })

                    // Save it to the library
                    .map(|a| {
                        self.local_library.set_artist(&a);
                        LibraryEntity::Artist(a)
                    })

                    // And return the result
                    .unwrap()
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
}



