use std::sync::mpsc::Receiver;

use dimple_core::{library::{LibrariesHandle, Library, LibraryHandle}, model::{Release, Image, Track}};
use dimple_sled_library::sled_library::SledLibrary;

#[derive(Debug)]
pub struct Librarian {
    local_library: SledLibrary,
    libraries: LibrariesHandle,
}

impl Default for Librarian {
    fn default() -> Self {
        Self { 
            local_library: SledLibrary::new("cache", "cache"),
            libraries: Default::default(), 
        }
    }
}

impl Librarian {
    pub fn add_library(&mut self, library: LibraryHandle) {
        self.libraries.write().unwrap().push(library.clone());
    }

    pub fn libraries(&self) -> LibrariesHandle {
        self.libraries.clone()
    }

    pub fn refresh_library(&self, library: &LibraryHandle) {
        for release in library.releases() {
            self.local_library.merge_release(library.as_ref(), &release).unwrap();
        }
    }

    pub fn refresh_all_libraries(&self) {
        let libraries = self.libraries.read().unwrap(); 
        for library in libraries.iter() {
            self.refresh_library(library);
        }
    }
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }
    
    fn releases(&self) -> Receiver<Release> {
        self.local_library.releases()
    }

    fn image(&self, image: &Image) -> Result<image::DynamicImage, String> {
        if let Ok(image) = self.local_library.image(image) {
            return Ok(image);
        }
        for library in self.libraries.read().unwrap().iter() {
            if let Ok(dynamic_image) = library.image(image) {
                return Ok(dynamic_image);
            }
        }
        Err("Not found".to_string())
    }

    fn stream(&self, track: &Track) -> Result<Vec<u8>, String> {
        if let Ok(stream) = self.local_library.stream(track) {
            return Ok(stream);
        }
        for library in self.libraries.read().unwrap().iter() {
            if let Ok(stream) = library.stream(track) {
                return Ok(stream);
            }
        }
        Err("Not found".to_string())
    }

    fn merge_release(&self, library: &dyn Library, release: &Release) 
        -> Result<(), String> {

        self.local_library.merge_release(library, release)
    }

}

