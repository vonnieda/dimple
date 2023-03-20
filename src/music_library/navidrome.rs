// TODO before I can do any id stuff I need to fix sunk to support
// string IDs. I need to do that so I can reference images by
// id.
// TODO I'm going to have to end up having, I guess, a complete database
// of all the data stored local for offline support, so might as well
// think about how to get the releases once and not over and over.
// But for now, I just wanna see images, so I should at least figure out
// how to cache them.
// TODO how does the UI change the credentials and other settings?
// Gonna need some generic UI thinger?

use std::{sync::{Mutex, Arc}, fs, time::Instant};

use image::DynamicImage;
use sunk::{search::SearchPage, Album, Client, ListType, Media};
use threadpool::ThreadPool;

use crate::MusicLibrary;

use super::Release;

// TODO Takes ~0.5ms 100x vs. 5-20ms for original. So, this is going to be
// super fun because we can load in the higher res ones dynnamically.
// It's about 19MB vs. 500MB
// Also, I bet if those loaded on the thread pool it would be super fast.
const CACHE_DIR: &str = "art/cache/navidrome/100x";

#[derive(Default)]
pub struct NavidromeMusicLibrary {
    site: String,
    username: String,
    password: String,
}

impl NavidromeMusicLibrary {
    pub fn new(site: &str, username: &str, password: &str) -> Self {
        Self {
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
        }
    }

    // fn sync(self: &Self) {
    //     if let Ok(client) = self.new_client() {
    //         if let Ok(new_releases) = get_all_releases(&client) {
    //             // TODO sync not extend
    //             self.releases.lock().unwrap().extend(new_releases);
    //         }
    //     }
    // }

    fn new_client(self: &Self) -> Result<Client, sunk::Error> {
        sunk::Client::new(
            self.site.as_str(),
            self.username.as_str(),
            self.password.as_str(),
        )
    }
}

impl MusicLibrary for NavidromeMusicLibrary {
    fn releases(self: &Self) -> Vec<Release> {
        if let Ok(client) = self.new_client() {
            if let Ok(releases) = get_all_releases(&client) {
                return releases;
            }
        }
        return Vec::new();
    }
}

fn get_all_releases(client: &Client) -> Result<Vec<Release>, sunk::Error> {
    let albums = get_all_albums(client)?;
    Ok(albums_to_releases(&albums, client))
}

fn albums_to_releases(albums: &Vec<Album>, client: &Client) -> Vec<Release> {
    let mut releases = Vec::new();
    for album in albums {
        let release = Release {
            title: album.name.clone(),
            artist: album.artist.clone(),
            cover_image: get_image(album, &client),
        };
        releases.push(release);
    }
    releases
}

fn get_image<M: Media>(media: &M, client: &Client) -> Option<DynamicImage> {
    if let Some(image) = load_image(media) {
        return Some(image);
    }
    else {
        if let Some(image) = download_image(media, client) {
            save_image(media, &image);
            return Some(image);
        }
        else {
            return None;
        }
    }
}

fn load_image<M: Media>(media: &M) -> Option<DynamicImage> {
    if let Some(cover_id) = media.cover_id() {
        let path = format!("{}/{}.png", CACHE_DIR, cover_id);
        if let Ok(image) = image::open(&path) {
            return Some(image);
        }
    }
    return None;
}

fn save_image<M: Media>(media: &M, image: &DynamicImage) {
    if let Some(cover_id) = media.cover_id() {
        let path = format!("{}/{}.png", CACHE_DIR, cover_id);
        let image_format = image::ImageFormat::Png;
        println!("Saving {}", path);
        match fs::create_dir_all(CACHE_DIR) {
            Ok(_) => {},
            Err(error) => eprintln!("Error: {}", error),
        }

        match image.save_with_format(path, image_format) {
            Ok(_) => return,
            Err(error) => eprintln!("Error: {}", error),
        }
    }
}

fn download_image<M: Media>(media: &M, client: &Client) -> Option<DynamicImage> {
    if let Some(cover_id) = media.cover_id() {
        if let Ok(cover_url) = media.cover_art_url(client, 0) {
            println!("Downloading {} from {}", cover_id, cover_url);
        }
    }
    if let Ok(image_data) = media.cover_art(client, 0) {
        if let Ok(image) = image::load_from_memory(&image_data) {
            return Some(image);
        }
    }
    return None;
}

fn get_all_albums(client: &Client) -> Result<Vec<Album>, sunk::Error> {
    let mut all_albums: Vec<Album> = Vec::new();
    let mut page = SearchPage {
        count: 500,
        offset: 0,
    };
    let mut albums = get_albums(page.count, page.offset, client)?;
    while albums.len() >= page.count {
        all_albums.extend(albums);
        page.offset += page.count;
        albums = get_albums(page.count, page.offset, client)?;
    }
    Ok(all_albums)
}

fn get_albums(count: usize, offset: usize, client: &Client) -> Result<Vec<Album>, sunk::Error> {
    println!("getting albums {} through {}", offset, offset + count - 1);
    let page = SearchPage { count, offset };
    let list_type = ListType::default();
    let albums = Album::list(&client, list_type, page, 0)?;
    Ok(albums)
}

