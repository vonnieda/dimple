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
// TODO it's possible that images are so special, because they are so
// heavy, that the main program DOES need to do the caching and I need
// to make the libraries support that.

use std::{fs, sync::mpsc::{channel, Sender, Receiver}, time::Instant};

use image::DynamicImage;
use sunk::{search::SearchPage, Album, Client, ListType, Media};
use threadpool::ThreadPool;

use crate::MusicLibrary;

use super::Release;

// TODO Takes ~0.5ms 100x vs. 5-20ms for original. So, this is going to be
// super fun because we can load in the higher res ones dynamically.
// It's about 19MB vs. 500MB
// Also, I bet if those loaded on the thread pool it would be super fast.
// Used mogrify -resize 200x *.png to resize
// const CACHE_DIR: &str = "art/cache/navidrome/50x"; // 94ms
// const CACHE_DIR: &str = "art/cache/navidrome/100x"; // 130ms
// const CACHE_DIR: &str = "art/cache/navidrome/200x"; // 211ms
const CACHE_DIR: &str = "data/navidrome/images/original"; // 1400ms

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

    // fn new_client(self: &Self) -> Result<Client, sunk::Error> {
    //     sunk::Client::new(
    //         self.site.as_str(),
    //         self.username.as_str(),
    //         self.password.as_str(),
    //     )
    // }

    fn new_client_info(self: &Self) -> ClientInfo {
        ClientInfo {
            site: self.site.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

impl MusicLibrary for NavidromeMusicLibrary {
    fn releases(self: &Self) -> Vec<Release> {
        if let Ok(releases) = get_all_releases(&self.new_client_info()) {
            return releases;
        }
        return Vec::new();
    }
}

#[derive(Default, Clone)]
struct ClientInfo {
    site: String,
    username: String,
    password: String,
}

fn get_all_releases(client_info: &ClientInfo) -> Result<Vec<Release>, sunk::Error> {
    let client = sunk::Client::new(
        client_info.site.as_str(),
        client_info.username.as_str(),
        client_info.password.as_str(),
    )?;
    let albums = get_all_albums(&client)?;
    return Ok(albums_to_releases(&albums, client_info));
}

fn albums_to_releases(albums: &Vec<Album>, client_info: &ClientInfo) -> Vec<Release> {
    let thread_pool = ThreadPool::new(4);
    let (tx, rx): (Sender<Release>, Receiver<Release>) = channel();
    for album in albums {
        let tx = tx.clone();
        let album = album.clone();
        let client_info = client_info.clone();
        thread_pool.execute(move || {
            let client = sunk::Client::new(
                client_info.site.as_str(),
                client_info.username.as_str(),
                client_info.password.as_str(),
            );

            if let Ok(client) = client {
                let release = Release {
                    id: album.id_string.clone(),
                    title: album.name.clone(),
                    artist: album.artist.clone(),
                    cover_image: get_image(&album, &client),
                    genre: album.genre.clone(),
                };
                tx.send(release).unwrap();
            }        
        });
    }
    thread_pool.join();
    // TODO Refactor messy
    let mut releases = Vec::new();
    loop {
        if let Ok(release) = rx.try_recv() {
            releases.push(release);
        }
        else {
            break;
        }
    }

    releases
}

// TODO add width param and auto resize and cache as needed
// TODO move all the caching stuff into a couple functions that
// other implementations can use. Probably just stick em in MusicLibrary.
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
    loop {
        if let Ok(albums) = get_albums(page.count, page.offset, client) {
            if albums.len() == 0 {
                break;
            }
            all_albums.extend(albums);
            page.offset += page.count;
        }
        else {
            break;
        }
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

