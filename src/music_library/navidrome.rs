// TODO before I can do any id stuff I need to fix sunk to support
// string IDs. I need to do that so I can reference images by
// id.

use std::{sync::{Arc, Mutex}};

use sunk::{search::SearchPage, Album, Client, ListType};
use threadpool::ThreadPool;

use crate::MusicLibrary;

use super::Release;

#[derive(Clone, Default)]
struct Auth {
    site: String,
    username: String,
    password: String,
}

pub struct NavidromeMusicLibrary {
    auth: Auth,

    releases: Arc<Mutex<Vec<NavidromeRelease>>>,

    thread_pool: ThreadPool,

    // image_pool: Arc<Mutex<HashMap<String, RetainedImage>>>,
}

impl NavidromeMusicLibrary {
    pub fn new(site: &str, username: &str, password: &str) -> Self {
        let selph = 
        Self {
            auth: Auth {
                site: String::from(site),
                username: String::from(username),
                password: String::from(password),
            },
            releases: Arc::new(Mutex::new(Vec::new())),
            thread_pool: ThreadPool::new(4),
            // image_pool: Arc::new(Mutex::new(HashMap::new())),
        };
        selph.sync();
        selph
    }

    pub fn sync(self: &Self) {
        // let releases_mutex = self.releases.clone();
        // let auth = self.auth.clone();
        // self.thread_pool.execute(move || {
        //     let releases = match Self::get_all_releases(&auth) {
        //         Ok(releases) => releases,
        //         Err(error) => {
        //             eprintln!("Error fetching releases: {}", error);
        //             return;
        //         }
        //     };
        //     let mut releases_guard = releases_mutex.lock().unwrap();
        //     releases_guard.extend(releases);
        // });

        let auth = self.auth.clone();
        let self_releases = self.releases.clone();
        self.thread_pool.execute(move || {
            if let Ok(releases) = Self::get_all_releases(&auth) {
                self_releases.lock().unwrap().extend(releases);
            }
        });
    }    

    fn get_all_releases(auth: &Auth) -> Result<Vec<NavidromeRelease>, sunk::Error> {
        let albums = Self::get_all_albums(auth)?;
        Ok(Self::albums_to_releases(&albums))
    }

    fn albums_to_releases(albums: &Vec<Album>) -> Vec<NavidromeRelease> {
        let mut releases = Vec::new();
        for album in albums {
            releases.push(Self::album_to_release(album));
        }
        releases
    }

    fn album_to_release(album: &Album) -> NavidromeRelease {
        // TODO go back up the call three from here and see I can get
        // rid of this one liner
        NavidromeRelease {
            album: album.clone()
        }
    }

    fn get_all_albums(auth: &Auth) -> Result<Vec<Album>, sunk::Error> {
        let mut all_albums: Vec<Album> = Vec::new();
        let mut page = SearchPage {
            count: 500,
            offset: 0,
        };
        let mut albums = Self::get_albums(page.count, page.offset, auth)?;
        while albums.len() >= page.count {
            all_albums.extend(albums);
            page.offset += page.count;
            albums = Self::get_albums(page.count, page.offset, auth)?;
        }
        Ok(all_albums)
    }

    fn get_albums(count: usize, offset: usize, auth: &Auth) -> Result<Vec<Album>, sunk::Error> {
        println!("getting albums {} through {}", offset, offset + count - 1);
        let client = Self::new_client(auth)?;
        let page = SearchPage { count, offset };
        let list_type = ListType::default();
        let albums = Album::list(&client, list_type, page, 0)?;
        Ok(albums)
    }

    fn new_client(auth: &Auth) -> Result<Client, sunk::Error> {
        sunk::Client::new(
            auth.site.as_str(),
            auth.username.as_str(),
            auth.password.as_str(),
        )
    }
}

impl MusicLibrary for NavidromeMusicLibrary {
    fn releases(self: &Self) -> Vec<Release> {
        if let Ok(releases) = self.releases.lock() {
            let mut ret:Vec<Release> = Vec::new();
            for release in releases.iter() {
                ret.push(Release {
                    title: release.album.name.clone(),
                    artist: release.album.artist.clone(),
                    release_year: None,
                    cover_image: None,
                });
            }
            ret
        }        
        else {
            Vec::new()
        }
    }
}


// _Album { 
//   id: "5649bff75a7b36d4789946f420712afa", 
//   name: "Freelance Bubblehead", 
//   artist: Some("1000 Clowns"), 
//   artist_id: Some("0e9ab60a6f701b3a727ef8d774bd00e1"), 
//   cover_art: Some("al-5649bff75a7b36d4789946f420712afa_63ebedbf"), 
//   song_count: 1, 
//   duration: 315, 
//   created: "2023-02-14T07:30:55.062040227Z", 
//   year: Some(1999), 
//   genre: None, 
//   song: [] }
#[derive(Clone)]
pub struct NavidromeRelease {
    album: Album,
}

