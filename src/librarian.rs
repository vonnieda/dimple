use std::{sync::{Arc, RwLock}};

use crossbeam::channel::{Receiver};

use crate::{music_library::{Library, Release, Image, Track, local_library::LocalLibrary, LibraryConfig, navidrome_library::NavidromeLibrary, memory_library::MemoryLibrary}, settings::Settings};

/// Manages a collection of Library and provides merging and caching for the
/// union of their releases.
#[derive(Debug)]
pub struct Librarian {
    memory_cache: MemoryLibrary,
    disk_cache: LocalLibrary,
    libraries: LibrariesHandle,
}

impl Default for Librarian {
    fn default() -> Self {
        Self { 
            memory_cache: Default::default(),
            disk_cache: LocalLibrary::new("cache", "cache"),
            libraries: Default::default(), 
        }
    }
}

type LibraryHandle = Arc<Box<dyn Library>>;

type LibrariesHandle = Arc<RwLock<Vec<LibraryHandle>>>;

impl Librarian {    
    pub fn add_library(&mut self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(Arc::new(library));
    }

    pub fn libraries(&self) -> LibrariesHandle {
        self.libraries.clone()
    }

    /// refresh merges all the remotes into the local and memory cache
    pub fn refresh(&self) {
        let libraries = self.libraries.read().unwrap(); 
        for library in libraries.iter() {
            let library = library.clone();
            log::info!("refreshing {}", library.name());
            for release in library.releases() {
                self.disk_cache.merge_release(library.as_ref().as_ref(), &release).unwrap();
            }
            log::info!("done refreshing {}", library.name());
        }
    }
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }
    
    // Okay, so maybe what happens here is on the first call we read the disk
    // cache into the memory cache, merging the data as we go using our merging
    // "algorithm". This is where de-duplication happens. I can either implement
    // it here or in the memorylibrary. Then we return the memory cache releases.
    // From then on, we assume the memory cache is up to date?
    fn releases(&self) -> Receiver<Release> {
        if self.memory_cache.releases_len() == 0 {
            log::info!("memory cache is dry, preloading from disk");
            for release in self.disk_cache.releases() {
                // TODO this is where de-dup happens
                self.memory_cache.merge_release(&self.disk_cache, &release).unwrap();
            }
            log::info!("done");
        }
        self.memory_cache.releases()
    }

    fn image(&self, image: &Image) -> Result<image::DynamicImage, String> {
        if let Ok(image) = self.memory_cache.image(image) {
            return Ok(image);
        }
        if let Ok(image) = self.disk_cache.image(image) {
            return Ok(image);
        }
        for library in self.libraries.read().unwrap().iter() {
            if let Ok(dynamic_image) = library.image(image) {
                self.memory_cache.merge_image(image, &dynamic_image);
                return Ok(dynamic_image);
            }
        }
        Err("Not found".to_string())
    }

    /// Check the in memory cache, followed by the libraries.
    fn stream(&self, track: &Track, sink: &rodio::Sink) -> Result<(), String>{
        if self.memory_cache.stream(track, sink).is_ok() {
            return Ok(());
        }
        if self.disk_cache.stream(track, sink).is_ok() {
            return Ok(());
        }
        for library in self.libraries.read().unwrap().iter() {
            if library.stream(track, sink).is_ok() {
                return Ok(());
            }
        }
        Err("Not found".to_string())
    }
}

impl From<Vec<LibraryConfig>> for Librarian {
    fn from(configs: Vec<LibraryConfig>) -> Self {
        let mut librarian = Self::default();
        for config in configs {
            let library: Box<dyn Library> = match config {
                LibraryConfig::Navidrome(config) => Box::new(NavidromeLibrary::from(config)),
                LibraryConfig::Local(config) => Box::new(LocalLibrary::from(config)),
            };
            librarian.add_library(library)
        }
        librarian
    }
}

impl From<Settings> for Librarian {
    fn from(settings: Settings) -> Self {
        Librarian::from(settings.libraries)
    }
}


