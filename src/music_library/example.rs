// use std::{sync::Arc, fs::File, io::Read};

// use egui_extras::RetainedImage;

// use rand::prelude::*;

// use crate::MusicLibrary;

// use super::Release;

// #[derive(Default)]
// pub struct ExampleMusicLibrary {
//     releases: Vec<Release>,

//     images: Vec<Arc<RetainedImage>>,
// }

// impl ExampleMusicLibrary {
//     pub fn new() -> Self {
//         let mut music_library = Self::default();
//         music_library.images = load_sample_images();
//         music_library.releases = create_sample_releases(1000, &music_library);

//         return music_library;
//     }
// }

// impl MusicLibrary for ExampleMusicLibrary {
//     fn ping(self: &Self) -> Result<(), String> {
//         Ok(())
//     }

//     fn releases(self: &Self) -> &[Release] {
//         &self.releases
//     }
// }

// fn load_sample_images() -> Vec<Arc<RetainedImage>> {
//     let filenames = [
//         "art/samples/getCoverArt-0.jpg",
//         "art/samples/getCoverArt-1.jpg",
//         "art/samples/getCoverArt-2.jpg",
//         "art/samples/getCoverArt-3.jpg",
//         "art/samples/getCoverArt-4.jpg",
//         "art/samples/getCoverArt-5.jpg",
//         "art/samples/getCoverArt-6.jpg",
//         "art/samples/getCoverArt-7.jpg",
//         "art/samples/getCoverArt-8.jpg",
//         "art/samples/getCoverArt-9.jpg",
//     ];
//     let mut images = Vec::new();
//     for filename in filenames {
//         if let Ok(image) = load_image(filename) {
//             images.push(Arc::new(image));
//         }
//     }
//     return images;
// }

// fn load_image(filename: &str) -> Result<RetainedImage, String> {
//     // TODO handle error
//     let mut f = File::open(filename).unwrap();
//     let mut buffer = Vec::new();
//     f.read_to_end(&mut buffer).unwrap();
//     return RetainedImage::from_image_bytes(filename, &buffer[..]);
// }

// fn create_sample_releases(count: i32, music_library: &ExampleMusicLibrary) -> Vec<Release> {
//     let mut releases = Vec::new();
//     for _ in 0..count {
//         releases.push(create_sample_release(music_library));
//     }
//     return releases;
// }

// fn create_sample_release(music_library: &ExampleMusicLibrary) -> Release {
//     let mut rng = rand::thread_rng();
//     Release {
//         artist: generate_artist_name(),
//         title: generate_album_name(),
//         release_year: 1930 + (rng.gen::<u32>() % 83),
//         cover_image: music_library.images.choose(&mut rng).cloned(),
//     }
// }

// fn generate_artist_name() -> String {
//     let artist_words = [
//         "The", "Open", "Wild", "Final", "Last", "Alaska", "Bear", "Beach", "Time", "Animal",
//     ];

//     let mut rng = rand::thread_rng();
//     let mut shuffled_words = artist_words.to_vec();
//     shuffled_words.shuffle(&mut rng);

//     let album = shuffled_words[..2].join(" ");
//     return album;
// }

// fn generate_album_name() -> String {
//     let album_words = [
//         "A", "Place", "For", "Children", "Walking", "Towards", "Bright", "Signs", "Stars", "Body",
//         "Music", "Songs", "Deaf", "Blind",
//     ];

//     let mut rng = rand::thread_rng();
//     let mut shuffled_words = album_words.to_vec();
//     shuffled_words.shuffle(&mut rng);

//     let album = shuffled_words[..3].join(" ");
//     return album;
// }

// // fn generate_sample_image() -> Arc<RetainedImage> {
// //     return Arc::new(RetainedImage::from_color_image("debug_name", ColorImage::example()));
// // }
