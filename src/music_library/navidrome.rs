use std::sync::Arc;

use egui_extras::RetainedImage;
use sunk::{search::SearchPage, ListType, Album, Media};

use crate::MusicLibrary;

use super::Release;

pub struct NavidromeMusicLibrary {
    site: String,
    username: String,
    password: String,

    releases: Vec<Release>,
}

impl NavidromeMusicLibrary {
    pub fn new(site: &str, username: &str, password: &str) -> Self {
        let mut s = Self {
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
            releases: Vec::new(),
        };

        Self::load_releases(&mut s);

        return s;
    }

    pub fn load_releases(self: &mut Self) {
        // TODO error unwrap, and also this is all trash
        let client = sunk::Client::new(
            self.site.as_str(), 
            self.username.as_str(),
            self.password.as_str()).unwrap();
        let page = SearchPage {
            count: 5,
            offset: 0,
        };
        let list_type = ListType::default();
        if let Ok(albums) = Album::list(&client, list_type, page, 0) {
            for album in albums {
                let mut release = Release::default();
                release.title = album.name.clone();
                if let Ok(url) = album.cover_art_url(&client, 200) {
                    println!("Downloading {}", url);
                }
                if let Ok(cover_art) = album.cover_art(&client, 200) {
                    if let Ok(image) = RetainedImage::from_image_bytes("art", &cover_art) {
                        release.cover_image = Some(Arc::new(image));
                    }
                }
                if let Some(artist) = album.artist {
                    release.artist = artist.clone();
                }
                if let Some(year) = album.year {
                    release.release_year = year as u32;
                }
                self.releases.push(release);
            }
        }
    }
}

impl MusicLibrary for NavidromeMusicLibrary {
    fn ping(self: &Self) -> Result<(), String> {
        Ok(())
    }

    fn releases(self: &Self) -> &[Release] {
        &self.releases
    }
}
