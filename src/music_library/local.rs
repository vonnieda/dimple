// use std::sync::Arc;

// use image::DynamicImage;
// /// A local music library living in a directory. Stores data with Sled.
// /// Faster than remote, but slower than memory. This is how the app stores
// /// the combined library from all the remotes.

// use rayon::prelude::*;
// use serde::{Deserialize, Serialize};
// use sled::Tree;
// use super::{Release, image_cache::ImageCache, Library, Image};

// pub struct LocalMusicLibrary {
//     releases: Tree,
//     images: Arc<ImageCache>,
//     audio: Tree,
// }

// impl LocalMusicLibrary {
//     pub fn new(path: &str) -> Self {
//         let db = sled::open(path).unwrap();
//         let releases = db.open_tree("releases").unwrap();
//         let images = db.open_tree("images").unwrap();
//         let audio = db.open_tree("audio").unwrap();
//         Self { 
//             releases,
//             images: Arc::new(ImageCache::new(images)),
//             audio,
//         }
//     }
// }

// /// TODO I don't like thow this is turning out. It feels heavy and fragile.
// /// Could be that it just needs to be broken up into more classes and functions
// /// but it already feels overwrought.
// /// I think I don't like the bincode stuff.
// /// 
// impl Library for LocalMusicLibrary {
//     fn releases(&self) -> Result<Vec<Release>, String> {
//         // let internal_releases: Vec<InternalRelease> = self.releases
//         //     .iter()
//         //     .par_bridge()
//         //     .map(|kv| {
//         //         let (_key, bin) = kv.unwrap();
//         //         bincode::deserialize(&bin[..]).unwrap()
//         //     })
//         //     .collect();

//         // // TODO I think this can be done further in parallel by doing the
//         // // deserialization here.
//         // // TODO removed the parallization here temporary cause I can't get it
//         // // to work with all my custom types.
//         // let releases: Vec<Arc<Release>> = internal_releases
//         //     .iter()
//         //     // .par_iter()
//         //     .map(|internal_release| Arc::new(Release {
//         //         id: internal_release.id.clone(),
//         //         title: internal_release.title.clone(),
//         //         artist: internal_release.artist.clone(),
//         //         cover_art: None,
//         //         genre: internal_release.genre.clone(),
//         //         tracks: Default::default(),
//         //     }))
//         //     .collect();

//         // return releases;
//         Ok(Default::default())
//     }

//     fn merge_release(self: &Self, release: &Release) -> Result<(), String> {
//         // If there is cover art, store it.
//         // let cover_art_id = release.cover_art.as_ref()
//         //     .map_or(None, |cover_art| cover_art.original())
//         //     .map_or(None, |original| {
//         //         self.images.insert(&release.id, &original);
//         //         return Some(release.id.clone());
//         // });

//         // Store the tracks and create InternalTrack objects for each
//         // TODO parallel download
//         // let internal_tracks = release.tracks
//         //     .iter()
//         //     // Filter out any tracks that don't have a stream
//         //     .filter_map(|track| {
//         //         return match &track.stream {
//         //             Some(stream) => Some((track, stream)),
//         //             None => None
//         //         };
//         //     })
//         //     // Store the stream and create an InternalTrack to represent it
//         //     .map(|(track, stream)| {
//         //         // TODO YOU FOOL! Someone is gonna have the same track title.
//         //         // Will fix when we have more track data.
//         //         let id = format!("{}:{}", release.id, track.title);
//         //         println!("Downloading {} {}", release.title, track.title);
//         //         self.audio.insert(id, stream.stream()).expect("self.audio.insert");
//         //         return Some(InternalTrack { title: track.title.clone() });
//         //     })
//         //     // Clear out any that failed
//         //     .filter_map(|x| x)
//         //     .collect();

//         // Create serializable version.
//         let internal_release = InternalRelease {
//             id: release.id.clone(),
//             title: release.title.clone(),
//             artist: release.artist.clone(),
//             cover_image_id: Default::default(),
//             genre: release.genre.clone(),
//             tracks: Default::default(),
//         };

//         // Store the release.
//         if let Ok(bin) = bincode::serialize(&internal_release) {
//             self.releases.insert(&release.id, bin).expect("insert failed");
//         }

//         Ok(())
//     }
// }

// #[derive(Clone)]
// struct LocalScaledImage {
//     images: Arc<ImageCache>,
//     id: String,
// }

// // impl Image for LocalScaledImage {
// //     fn scaled(&self, width: u32, height: u32) -> Option<DynamicImage> {
// //         self.images.get(&self.id, width, height)
// //     }

// //     fn original(&self) -> Option<DynamicImage> {
// //         self.images.get_original(&self.id)
// //     }
// // }

// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// struct LocalStream {
//     id: String,
// }

// // impl Stream for LocalStream {
// //     fn stream(&self) -> Vec<u8> {
// //         todo!()
// //     }
// // }

// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// struct InternalRelease {
//     id: String,
//     title: String,
//     artist: Option<String>,
//     cover_image_id: Option<String>,
//     genre: Option<String>,
//     tracks: Vec<InternalTrack>,
// }

// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// struct InternalTrack {
//     title: String,    
// }