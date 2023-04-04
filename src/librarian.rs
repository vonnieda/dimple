use std::{sync::{Arc, RwLock}};

use crossbeam::channel::{unbounded, Receiver};

use crate::music_library::{Library, Release, Image, Track, local::LocalLibrary};

/// Librarian manages a local library that is used for caching and a list of
/// libraries that are used as sources. 
pub struct Librarian {
    cache: LocalLibrary,
    libraries: Arc<RwLock<Vec<LibraryHandle>>>,
}

type LibraryHandle = Arc<Box<dyn Library>>;

impl Librarian {    
    pub fn new() -> Self {
        let libraries: Arc<RwLock<Vec<LibraryHandle>>> = Default::default();

        Self {
            // TODO ulid
            cache: LocalLibrary::new("cache", "cache"),
            libraries,
        }
    }

    pub fn add_library(&mut self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(Arc::new(library));
    }
}

impl Default for Librarian {
    fn default() -> Self {
        Self::new()
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
        self.cache.merge_release(library, release)
    }
}

