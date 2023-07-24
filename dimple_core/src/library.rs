use std::{sync::{mpsc::Receiver, Arc, RwLock}, collections::HashSet, fmt::Debug};

use image::DynamicImage;

use crate::model::{Release, Image, Track, Artist, Genre};

/// Objects all have a URL. The URL must uniquely identify that object and
/// it should, at least, have a scheme that matches the library it
/// came from. In other words, given the same library config, it should be
/// possible to re-load the object from the library with the same URL.

pub type LibraryHandle = Arc<dyn Library>;

pub type LibrariesHandle = Arc<RwLock<Vec<LibraryHandle>>>;

pub trait Library: Send + Sync {
    fn name(&self) -> String;

    fn releases(&self) -> Receiver<Release>;

    fn image(&self, _image: &Image) -> Result<DynamicImage, String>;

    // TODO I wanted to have this return a Source but I couldn't figure out how.
    fn stream(&self, _track: &Track) -> Result<Vec<u8>, String>;

    fn merge_release(&self, _library: &dyn Library, _release: &Release) -> Result<(), String> {
        todo!();
    }

    fn releases_by_artist(&self, artist: &Artist) -> Vec<Release> {
        let mut releases = self.releases()
            .iter()
            .filter(|release| release.artists.contains(artist))
            .collect::<Vec<Release>>();
        self.sort_releases(&mut releases);
        releases
    }

    fn releases_by_genre(&self, genre: &Genre) -> Vec<Release> {
        let mut releases = self.releases()
            .into_iter()
            .filter(|release| release.genres.contains(genre))
            .collect::<Vec<Release>>();
        self.sort_releases(&mut releases);
        releases
    }

    fn artists(&self) -> Vec<Artist> {
        let mut artists = self.releases()
            .iter()
            .flat_map(|release| release.artists.into_iter())
            .collect::<HashSet<Artist>>()
            .into_iter()
            .collect::<Vec<Artist>>();
        self.sort_artists(&mut artists);
        artists
    }

    fn artists_by_genre(&self, genre: &Genre) -> Vec<Artist> {
        let mut artists = self.releases_by_genre(genre)
            .into_iter()
            .flat_map(|release| release.artists.into_iter())
            .collect::<HashSet<Artist>>()
            .into_iter()
            .collect::<Vec<Artist>>();
        self.sort_artists(&mut artists);
        artists
    }

    fn genres(&self) -> Vec<Genre> {
        let mut genres = self.releases()
            .iter()
            .flat_map(|release| release.genres.into_iter())
            .collect::<HashSet<Genre>>()
            .into_iter()
            .collect::<Vec<Genre>>();
        self.sort_genres(&mut genres);
        genres
    }

    fn genres_by_artist(&self, artist: &Artist) -> Vec<Genre> {
        let mut genres = self.releases_by_artist(artist)
            .into_iter()
            .flat_map(|release| release.genres.into_iter())
            .collect::<HashSet<Genre>>()
            .into_iter()
            .collect::<Vec<Genre>>();
        self.sort_genres(&mut genres);
        genres
    }

    fn similar_artists(&self, artist: &Artist) -> Vec<Artist> {
        let mut artists = self.genres_by_artist(artist)
            .into_iter()
            .flat_map(|genre| self.artists_by_genre(&genre).into_iter())
            .collect::<HashSet<Artist>>()
            .into_iter()
            .collect::<Vec<Artist>>();
        self.sort_artists(&mut artists);
        artists
    }

    fn similar_genres(&self, genre: &Genre) -> Vec<Genre> {
        let mut genres = self.artists_by_genre(genre)
            .into_iter()
            .flat_map(|artist| self.genres_by_artist(&artist).into_iter())
            .collect::<HashSet<Genre>>()
            .into_iter()
            .collect::<Vec<Genre>>();
        self.sort_genres(&mut genres);
        genres
    }

    // TODO Change to a comparator
    fn sort_artists(&self, artists: &mut[Artist]) {
        artists.sort_by(|a, b| {
            a.name.to_lowercase()
                .cmp(&b.name.to_lowercase())
        });
    }

    fn sort_genres(&self, genres: &mut[Genre]) {
        genres.sort_by(|a, b| {
            a.name.to_lowercase()
                .cmp(&b.name.to_lowercase())
        });
    }

    fn sort_releases(&self, releases: &mut[Release]) {
        releases.sort_by(|a, b| {
            a.artist().to_lowercase()
                .cmp(&b.artist().to_lowercase())
                .then(a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
    }
}

impl Debug for dyn Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}
