use std::{sync::{Arc, RwLock}, collections::HashSet};

use crossbeam::channel::{Receiver};

use crate::{music_library::{Library, Release, Image, Track, local_library::LocalLibrary, LibraryConfig, navidrome_library::NavidromeLibrary, Artist, Genre}, settings::Settings};

#[derive(Debug)]
pub struct Librarian {
    disk_cache: LocalLibrary,
    libraries: LibrariesHandle,
}

impl Default for Librarian {
    fn default() -> Self {
        Self { 
            disk_cache: LocalLibrary::new("cache", "cache"),
            libraries: Default::default(), 
        }
    }
}

type LibraryHandle = Arc<Box<dyn Library>>;

type LibrariesHandle = Arc<RwLock<Vec<LibraryHandle>>>;

impl Librarian {
    pub fn add_library(&mut self, library: Box<dyn Library>) {
        let library = Arc::new(library);
        self.libraries.write().unwrap().push(library.clone());
    }

    pub fn libraries(&self) -> LibrariesHandle {
        self.libraries.clone()
    }

    pub fn refresh_library(&self, library: &LibraryHandle) {
        for release in library.releases() {
            self.disk_cache.merge_release(library.as_ref().as_ref(), &release).unwrap();
        }
    }

    pub fn refresh_all_libraries(&self) {
        let libraries = self.libraries.read().unwrap(); 
        for library in libraries.iter() {
            self.refresh_library(library);
        }
    }

    /// TODO all these shortcut methods might end up in Library since they apply
    /// there too, and may be more efficient being called directly. Defaults
    /// can just be these ones that filter.
    pub fn artists(&self) -> Vec<Artist> {
        self.releases()
            .iter()
            .flat_map(|release| release.artists.into_iter())
            .collect::<Vec<Artist>>()
    }

    pub fn artists_by_genre(&self, genre: &Genre) -> Vec<Artist> {
        Vec::new()
    }

    pub fn genres(&self) -> Vec<Genre> {
        Vec::new()
    }

    pub fn genres_by_artist(&self, artist: &Artist) -> Vec<Genre> {
        Vec::new()
    }

    pub fn releases_by_artist(&self, artist: &Artist) -> Vec<Release> {
        self.releases()
            .into_iter()
            .filter(|release| release.artists.contains(artist))
            .collect()
    }

    pub fn releases_by_genre(&self, genre: &Genre) -> Vec<Release> {
        self.releases()
            .into_iter()
            .filter(|release| release.genres.contains(genre))
            .collect()
    }
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }
    
    fn releases(&self) -> Receiver<Release> {
        self.disk_cache.releases()
    }

    fn image(&self, image: &Image) -> Result<image::DynamicImage, String> {
        if let Ok(image) = self.disk_cache.image(image) {
            return Ok(image);
        }
        for library in self.libraries.read().unwrap().iter() {
            if let Ok(dynamic_image) = library.image(image) {
                return Ok(dynamic_image);
            }
        }
        Err("Not found".to_string())
    }

    fn stream(&self, track: &Track, sink: &rodio::Sink) -> Result<(), String>{
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

    fn merge_release(&self, library: &dyn Library, release: &Release) 
        -> Result<(), String> {

        self.disk_cache.merge_release(library, release)
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


