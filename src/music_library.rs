use egui_extras::RetainedImage;
use rand::prelude::*;
use sunk::search::SearchPage;
use sunk::{Album, ListType, search, Media};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use rand::seq::SliceRandom;

// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs

#[derive(Default)]
pub struct MusicLibrary {
    pub releases: Vec<Release>,
    pub playlists: Vec<Playlist>,
    pub images: Vec<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Playlist {
    pub name: String,
    pub items: Vec<Release>,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Release {
    pub title: String,
    pub artist: String,
    pub release_year: u32,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Artist {
    pub name: String,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Track {
    pub title: String,
    pub lyrics: Lyrics,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Lyrics {
    pub lyrics: String,
}

impl MusicLibrary {
    pub fn from_navidrome(site: &str, username: &str, password: &str) -> Self {
        let mut music_library = Self::default();

        music_library.images = load_sample_images();
       
        // TODO error unwrap
        let client = sunk::Client::new(site, username, password).unwrap();
        println!("ping? {:?}", client.ping());
        if let Ok(albums) = Album::list(&client, ListType::default(), SearchPage { count : 40, offset : 0 }, 0) {
            for album in albums {
                let mut release = Release::default();
                release.title = album.name.clone();
                if let Ok(cover_art) = album.cover_art(&client, 0) {
                    if let Ok(image) = RetainedImage::from_image_bytes("art", &cover_art) {
                        release.cover_image = Some(Arc::new(image));
                    }
                }
                // TODO else default image? or handle at the rendering point?
                if let Some(artist) = album.artist {
                    release.artist = artist.clone();
                }
                if let Some(year) = album.year {
                    release.release_year = year as u32;
                }
                
                music_library.releases.push(release);
            }
        }

        return music_library;
    }

    pub fn example() -> Self {
        let _num_artists = 706;
        let num_releases = 1371;

        let mut music_library = Self::default();
        music_library.images = load_sample_images();
        music_library.releases = create_sample_releases(num_releases, &music_library);

        return music_library;
    }
}

fn load_sample_images() -> Vec<Arc<RetainedImage>> {
    let filenames = [
        "art/samples/getCoverArt-0.jpg",
        "art/samples/getCoverArt-1.jpg",
        "art/samples/getCoverArt-2.jpg",
        "art/samples/getCoverArt-3.jpg",
        "art/samples/getCoverArt-4.jpg",
        "art/samples/getCoverArt-5.jpg",
        "art/samples/getCoverArt-6.jpg",
        "art/samples/getCoverArt-7.jpg",
        "art/samples/getCoverArt-8.jpg",
        "art/samples/getCoverArt-9.jpg",
    ];
    let mut images = Vec::new();
    for filename in filenames {
        if let Ok(image) = load_image(filename) {
            images.push(Arc::new(image));
        }
    }
    return images;
}

fn load_image(filename: &str) -> Result<RetainedImage, String> {
    // TODO handle error
    let mut f = File::open(filename).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    return RetainedImage::from_image_bytes(
        filename, 
        &buffer[..]);
}

fn create_sample_releases(count: i32, music_library: &MusicLibrary) -> Vec<Release> {
    let mut releases = Vec::new();
    for _ in 0..count {
        releases.push(create_sample_release(music_library));
    }
    return releases;
}

fn create_sample_release(music_library: &MusicLibrary) -> Release {
    let mut rng = rand::thread_rng();
    Release {
        artist: generate_artist_name(),
        title: generate_album_name(),
        release_year: 1930 + (rng.gen::<u32>() % 83),
        cover_image: music_library.images.choose(&mut rng).cloned(),
    }
}

fn generate_artist_name() -> String {
    let artist_words = [
        "The",
        "Open",
        "Wild",
        "Final",
        "Last",
        "Alaska",
        "Bear",
        "Beach",
        "Time",
        "Animal",
    ];

    let mut rng = rand::thread_rng();
    let mut shuffled_words = artist_words.to_vec();
    shuffled_words.shuffle(&mut rng);
    
    let album = shuffled_words[..2].join(" ");
    return album;
}

fn generate_album_name() -> String {
    let album_words = [
        "A",
        "Place",
        "For",
        "Children",
        "Walking",
        "Towards",
        "Bright",
        "Signs",
        "Stars",
        "Body",
        "Music",
        "Songs",
        "Deaf",
        "Blind",
    ];


    let mut rng = rand::thread_rng();
    let mut shuffled_words = album_words.to_vec();
    shuffled_words.shuffle(&mut rng);
    
    let album = shuffled_words[..3].join(" ");
    return album;
}
    
// fn generate_sample_image() -> Arc<RetainedImage> {
//     return Arc::new(RetainedImage::from_color_image("debug_name", ColorImage::example()));
// }
