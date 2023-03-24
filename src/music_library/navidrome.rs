
//     if false {
//         // load a remote music library
//         let remote_library:Box<dyn MusicLibrary> = match builder.build() {
//             Ok(config) => {
//                 Box::new(NavidromeMusicLibrary::new(
//                     config.get_string("navidrome.site").unwrap().as_str(),
//                     config.get_string("navidrome.username").unwrap().as_str(),
//                     config.get_string("navidrome.password").unwrap().as_str()))
//             },
//             Err(_) => {
//                 Box::new(EmptyMusicLibrary::default())
//             }
//         };
//         println!("Loading remote library");
//         let releases = remote_library.releases();
//         println!("Remote library contains {} releases", releases.len());

//         // merge all the remote releases into the local
//         for (i, release) in releases.iter().enumerate() {
//             println!("Merging {}/{}: {}", i + 1, releases.len(), release.title);
//             library.merge_release(&release).expect("merge error");
//         }
//     }


use std::{fs, sync::Arc};

use image::DynamicImage;
use rayon::prelude::*;
use sunk::{search::SearchPage, Album, Client, ListType, Media};

use super::{Release, MusicLibrary};

const CACHE_DIR: &str = "data/navidrome/images/original";

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

    fn new_client_info(self: &Self) -> ClientInfo {
        ClientInfo {
            site: self.site.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

impl MusicLibrary for NavidromeMusicLibrary {
    fn releases(self: &Self) -> Vec<Arc<Release>> {
        if let Ok(releases) = get_all_releases(&self.new_client_info()) {
            return releases.into_iter().map(|release| {
                Arc::new(release)
            }).collect();
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
    albums.par_iter().map(|album| {
        let client = sunk::Client::new(
            client_info.site.as_str(),
            client_info.username.as_str(),
            client_info.password.as_str(),
        ).unwrap();
        Release {
            id: album.id_string.clone(),
            title: album.name.clone(),
            artist: album.artist.clone(),
            cover_art: get_image(album, &client),
            genre: album.genre.clone(),
            tracks: Vec::new(),
        }
    }).collect()
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

