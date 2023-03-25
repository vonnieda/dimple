use std::{fs, sync::Arc};

use config::Config;
use image::{DynamicImage, imageops::FilterType};
use log::debug;
use rayon::prelude::*;
use sunk::{search::SearchPage, Album, Client, ListType, Media};

use super::{Library, Release, Image};

const CACHE_DIR: &str = "data/navidrome/images/original";

pub struct NavidromeLibrary {
    // TODO add path, or just take a sled?
    site: String,
    username: String,
    password: String,
}

impl NavidromeLibrary {
    pub fn new(site: &str, username: &str, password: &str) -> Self {
        Self {
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            site: config.get_string("navidrome.site").unwrap(),
            username: config.get_string("navidrome.username").unwrap(),
            password: config.get_string("navidrome.password").unwrap(),
        }
    }

    fn new_client(&self) -> Result<Client, String> {
        sunk::Client::new(
            self.site.as_str(),
            self.username.as_str(),
            self.password.as_str(),
        ).map_err(|err| err.to_string())
    }

}

impl Library for NavidromeLibrary {
    fn releases(self: &Self) -> Result<Vec<Release>, String> {
        let client = self.new_client()?;
        get_all_releases(&client).map_err(|x| x.to_string())
    }
}

fn get_all_releases(client: &Client) -> Result<Vec<Release>, sunk::Error> {
    let albums = get_all_albums(&client)?;
    return Ok(albums_to_releases(&albums, client));
}

fn albums_to_releases(albums: &Vec<Album>, client: &Client) -> Vec<Release> {
    albums
        .par_iter()
        .map(|album| {
            Release {
                id: album.id_string.clone(),
                title: album.name.clone(),
                artist: album.artist.clone(),
                cover_art: get_image(album, client),
                genre: album.genre.clone(),
                tracks: Default::default(),
            }
        }).collect()
}

// struct NavidromeImage {
//     client_info: ClientInfo,
//     // TODO it's late and I cannot figure out how to make this generic.
//     media: Album,
// }

// impl Image for NavidromeImage {
//     fn scaled(&self, width: u32, height: u32) -> Option<DynamicImage> {
//         self.original().map_or(None, |original| {
//             Some(original.resize(width, height, FilterType::CatmullRom))
//         })
//     }

//     fn original(&self) -> Option<DynamicImage> {
//         if let Ok(client) = Client::new(&self.client_info.site, 
//             &self.client_info.username, 
//             &self.client_info.password) {
//             return get_image(&self.media, &client);
//         }
//         None
//     }
// }

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
        debug!("Saving {}", path);
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
            debug!("Downloading {} from {}", cover_id, cover_url);
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
    debug!("getting albums {} through {}", offset, offset + count - 1);
    let page = SearchPage { count, offset };
    let list_type = ListType::default();
    let albums = Album::list(&client, list_type, page, 0)?;
    Ok(albums)
}

