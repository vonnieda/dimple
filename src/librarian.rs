use std::{sync::{Arc, RwLock, mpsc::Receiver}, collections::HashSet};

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

    pub fn releases_by_artist(&self, artist: &Artist) -> Vec<Release> {
        let mut releases = self.releases()
            .iter()
            .filter(|release| release.artists.contains(artist))
            .collect::<Vec<Release>>();
        Self::sort_releases(&mut releases);
        releases
    }

    pub fn releases_by_genre(&self, genre: &Genre) -> Vec<Release> {
        let mut releases = self.releases()
            .into_iter()
            .filter(|release| release.genres.contains(genre))
            .collect::<Vec<Release>>();
        Self::sort_releases(&mut releases);
        releases
    }

    pub fn artists(&self) -> Vec<Artist> {
        let mut artists = self.releases()
            .iter()
            .flat_map(|release| release.artists.into_iter())
            .collect::<Vec<Artist>>();
        Self::sort_artists(&mut artists);
        artists
    }

    pub fn artists_by_genre(&self, genre: &Genre) -> Vec<Artist> {
        let mut artists = self.releases_by_genre(genre)
            .into_iter()
            .flat_map(|release| release.artists.into_iter())
            .collect::<HashSet<Artist>>()
            .into_iter()
            .collect::<Vec<Artist>>();
        Self::sort_artists(&mut artists);
        artists
    }

    pub fn genres(&self) -> Vec<Genre> {
        let mut genres = self.releases()
            .iter()
            .flat_map(|release| release.genres.into_iter())
            .collect::<HashSet<Genre>>()
            .into_iter()
            .collect::<Vec<Genre>>();
        Self::sort_genres(&mut genres);
        genres
    }

    pub fn genres_by_artist(&self, artist: &Artist) -> Vec<Genre> {
        let mut genres = self.releases_by_artist(artist)
            .into_iter()
            .flat_map(|release| release.genres.into_iter())
            .collect::<HashSet<Genre>>()
            .into_iter()
            .collect::<Vec<Genre>>();
        Self::sort_genres(&mut genres);
        genres
    }

    pub fn similar_artists(&self, artist: &Artist) -> Vec<Artist> {
        let mut artists = self.genres_by_artist(&artist)
            .into_iter()
            .flat_map(|genre| self.artists_by_genre(&genre).into_iter())
            .collect::<HashSet<Artist>>()
            .into_iter()
            .collect::<Vec<Artist>>();
        Self::sort_artists(&mut artists);
        artists
    }

    pub fn similar_genres(&self, genre: &Genre) -> Vec<Genre> {
        let mut genres = self.artists_by_genre(&genre)
            .into_iter()
            .flat_map(|artist| self.genres_by_artist(&artist).into_iter())
            .collect::<HashSet<Genre>>()
            .into_iter()
            .collect::<Vec<Genre>>();
        Self::sort_genres(&mut genres);
        genres
    }

    // TODO Change to a comparator
    pub fn sort_artists(artists: &mut[Artist]) {
        artists.sort_by(|a, b| {
            a.name.to_lowercase()
                .cmp(&b.name.to_lowercase())
        });
    }

    pub fn sort_genres(genres: &mut[Genre]) {
        genres.sort_by(|a, b| {
            a.name.to_lowercase()
                .cmp(&b.name.to_lowercase())
        });
    }

    pub fn sort_releases(releases: &mut[Release]) {
        releases.sort_by(|a, b| {
            a.artist().to_lowercase()
                .cmp(&b.artist().to_lowercase())
                .then(a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
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

    fn stream(&self, track: &Track) -> Result<Vec<u8>, String> {
        if let Ok(stream) = self.disk_cache.stream(track) {
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


