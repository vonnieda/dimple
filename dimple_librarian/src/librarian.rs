use std::sync::RwLock;

use dimple_core::{library::{Library, LibraryEntity}, model::Artist};
use dimple_sled_library::sled_library::SledLibrary;

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

    pub fn merge_artist(src: &Artist, dest: &Artist) -> Artist {
        let mut dest = dest.clone();
        if dest.mbid.is_none() {
            dest.mbid = src.mbid.clone();
        }
        if dest.name.is_empty() {
            dest.name = src.name.clone();
        }
        dest
    }

    // Update or create the entity in the local library, returning the local
    // entity.
    fn resolve(&self, e: &LibraryEntity) -> LibraryEntity {
        log::info!("resolve {:?}", e);
        match e {
            LibraryEntity::Artist(a_in) => {
                self.local_library
                    .get_artist(&a_in.id)
                    .or_else(|| self.local_library.get_artist_by_mbid(a_in.mbid.clone()))
                    .or_else(|| Some(Artist::default()))
                    .map(|a| Self::merge_artist(a_in, &a))
                    .map(|a| {
                        self.local_library.set_artist(&a);
                        LibraryEntity::Artist(a)
                    })
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

    // fn list<T: dimple_core::library::LibraryEnt + 'static>(&self) -> Box<dyn Iterator<Item = T>> {
    //     Box::new(std::iter::empty())
    // }
}



