use std::sync::RwLock;

use dimple_core::{library::{Library, SearchResult}, model::Artist};
use dimple_sled_library::sled_library::SledLibrary;

pub struct Librarian {
    _local_library: SledLibrary,
    libraries: RwLock<Vec<Box<dyn Library>>>,
}

impl Default for Librarian {
    fn default() -> Self {
        Self { 
            _local_library: SledLibrary::new("local_library", "local_library"),
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

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = dimple_core::library::SearchResult>> {
        // TODO actually merge
        let merged: Vec<SearchResult> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| lib.search(query))
            .map(|result| {
                match result {
                    SearchResult::Artist(artist) => {
                        SearchResult::Artist(Artist {
                            musicbrainz_id: artist.musicbrainz_id,
                            name: artist.name,
                            ..Default::default()
                        })
                    }
                    SearchResult::Genre(_) => todo!(),
                    SearchResult::Release(_) => todo!(),
                    SearchResult::Track(_) => todo!(),
                }
            })
            // TODO figure out how to return the iterator directly instead of
            // first collecting it so they can start displaying asap.
            .collect();
        Box::new(merged.into_iter())
    }    
}



