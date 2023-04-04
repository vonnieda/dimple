use std::time::Instant;

use dimple::{music_library::{local::LocalLibrary, navidrome::{NavidromeLibrary}, Library, LibraryConfig}, dimple::Settings};

fn time_library(library: &dyn Library) {
    let t = Instant::now();
    let releases = library.releases();
    let mut count = 0;
    for (i, _release) in releases.iter().enumerate() {
        if i == 0 {
            log::info!("{}: {}ms to first release.", 
                library.name(),
                Instant::now().duration_since(t).as_millis());
        }
        if i % 250 == 0 {
            log::info!("{}: {}", 
                library.name(),
                i);
        }
        count += 1;
    }
    log::info!("{}: {}ms to release #{}, done.", 
        library.name(),
        Instant::now().duration_since(t).as_millis(), 
        count);
}

fn main() {
    // Load settings
    let config = config::Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()
        .unwrap();
    let settings: Settings = config.try_deserialize().unwrap();

    let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Info);
        builder.format_timestamp_millis();
        builder.init();

    for config in settings.libraries {
        let library: Box<dyn Library> = match config {
            LibraryConfig::Navidrome(config) => Box::new(NavidromeLibrary::from_config(config)) as Box<dyn Library>,
            LibraryConfig::Local(config) => Box::new(LocalLibrary::from_config(config)) as Box<dyn Library>,
        };
        time_library(library.as_ref());
    }
}



// pub struct Settings {
//     libraries: Vec<LibraryConfig>,
// }

// impl Dimple {
//     pub fn new(sink: Arc<Sink>) -> Self {
//         // Load settings
//         let config = config::Config::builder()
//             .add_source(config::File::with_name("config.yaml"))
//             .build()
//             .unwrap();
//         let settings: Settings = config.try_deserialize().unwrap();

//         // Create libraries from configs
//         let mut librarian = Librarian::new();
//         for config in settings.libraries {
//             let library = match config {
//                 Navidrome(config) => Box::new(NavidromeLibrary::from_config(config)) as Box<dyn Library>,
//                 Local(config) => Box::new(LocalLibrary::from_config(config)) as Box<dyn Library>,
//             };
//             librarian.add_library(library)
//         }
//         let librarian = Arc::new(librarian);

