use std::{sync::{Arc, RwLock}};

use crossbeam::channel::{unbounded, Receiver};

use crate::{music_library::{Library, Release, Image, Track, local::LocalLibrary, LibraryConfig, navidrome::NavidromeLibrary}, dimple::Settings};

/// Manages a collection of Library and provides merging and cachine for the
/// union of their releases.
#[derive(Debug)]
pub struct Librarian {
    libraries: LibrariesHandle,
}

type LibraryHandle = Arc<Box<dyn Library>>;

type LibrariesHandle = Arc<RwLock<Vec<LibraryHandle>>>;

impl Librarian {    
    pub fn new() -> Self {
        let libraries: LibrariesHandle = Default::default();

        Self {
            libraries,
        }
    }

    pub fn add_library(&mut self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(Arc::new(library));
    }

    pub fn libraries(&self) -> LibrariesHandle {
        self.libraries.clone()
    }
}

impl Default for Librarian {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<LibraryConfig>> for Librarian {
    fn from(configs: Vec<LibraryConfig>) -> Self {
        let mut librarian = Self::new();
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

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn releases(&self) -> Receiver<Release> {
        let (sender, receiver) = unbounded::<Release>();
        for library in self.libraries.read().unwrap().iter() {
            let sender = sender.clone();
            let library = library.clone();
            std::thread::spawn(move || {
                for release in library.releases() {
                    log::debug!("Loaded {} {}", library.name(), release.title);
                    sender.send(release.clone()).unwrap();
                }
            });
        }
        receiver
    }

    fn image(&self, image: &Image) -> Result<image::DynamicImage, String> {
        for library in self.libraries.read().unwrap().iter() {
            if let Ok(image) = library.image(image) {
                return Ok(image);
            }
        }
        Err("Not found".to_string())
    }

    fn stream(&self, track: &Track, sink: &rodio::Sink) -> Result<(), String>{
        for library in self.libraries.read().unwrap().iter() {
            if library.stream(track, sink).is_ok() {
                return Ok(());
            }
        }
        Err("Not found".to_string())
    }

    fn merge_release(&self, library: &dyn Library, release: &Release) -> Result<(), String> {
        todo!()
    }
}

