// use std::{time::Instant};

// use dimple_ui_egui::{music_library::{Library}, dimple::Settings, librarian::Librarian};

// fn time_library(library: &dyn Library) {
//     log::info!("{}: Testing", 
//         library.name());
//     let t = Instant::now();
//     let releases = library.releases();
//     let mut count = 0;
//     for (i, _release) in releases.iter().enumerate() {
//         if i == 0 {
//             log::info!("{}: {}ms to first release.", 
//                 library.name(),
//                 Instant::now().duration_since(t).as_millis());
//         }
//         if i % 250 == 0 {
//             log::info!("{}: {}", 
//                 library.name(),
//                 i);
//         }
//         count += 1;
//     }
//     log::info!("{}: {}ms to release #{}, done.", 
//         library.name(),
//         Instant::now().duration_since(t).as_millis(), 
//         count);
// }

// fn main() {
//     let mut builder = env_logger::Builder::new();
//     builder.filter_level(log::LevelFilter::Info);
//     builder.format_timestamp_millis();
//     builder.init();

//     let settings = Settings::default();
//     let librarian = Librarian::from(settings.libraries);

//     for library in librarian.libraries().read().unwrap().iter() {
//         time_library(library.as_ref().as_ref());
//     }
// }

