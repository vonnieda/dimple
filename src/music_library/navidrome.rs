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
        Self {
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
            releases: Vec::new(),
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


// pub fn from_navidrome(site: &str, username: &str, password: &str) -> Self {
//     let mut music_library = Self::default();

//     music_library.images = load_sample_images();

//     // TODO error unwrap
//     let client = sunk::Client::new(site, username, password).unwrap();
//     println!("ping? {:?}", client.ping());
//     let page = SearchPage {
//         count: 20,
//         offset: 0,
//     };
//     let list_type = ListType::default();
//     if let Ok(albums) = Album::list(&client, list_type, page, 0) {
//         for album in albums {
//             let mut release = Release::default();
//             release.title = album.name.clone();
//             if let Ok(url) = album.cover_art_url(&client, 0) {
//                 println!("Downloading {}", url);
//             }
//             if let Ok(cover_art) = album.cover_art(&client, 0) {
//                 if let Ok(image) = RetainedImage::from_image_bytes("art", &cover_art) {
//                     release.cover_image = Some(Arc::new(image));
//                 }
//             }
//             if let Some(artist) = album.artist {
//                 release.artist = artist.clone();
//             }
//             if let Some(year) = album.year {
//                 release.release_year = year as u32;
//             }

//             music_library.releases.push(release);
//         }
//     }

//     return music_library;
// }
