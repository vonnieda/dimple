use std::time::{Duration, Instant};

use dimple::music_library::{libraries::Libraries, local::LocalLibrary, navidrome::NavidromeLibrary, Library};

fn time_library(library: Box<dyn Library>) {
    let t = Instant::now();
    let releases = library.releases();
    let mut first = false;
    let mut count = 0;
    for _release in releases {
        if !first {
            first = true;
            log::info!("{}: {}ms to first release.", 
                library.name(),
                Instant::now().duration_since(t).as_millis());
        }
        count += 1;
    }
    log::info!("{}: {}ms to release #{}, done.", 
        library.name(),
        Instant::now().duration_since(t).as_millis(), 
        count);
}

fn main() {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build().expect("Config error");

    let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Info);
        builder.format_timestamp_millis();
        builder.init();

    time_library(Box::new(LocalLibrary::new("data/library")));
    time_library(Box::new(NavidromeLibrary::from_config(&config)));

    let mut libraries = Libraries::new();
    libraries.add_library(Box::new(LocalLibrary::new("data/library")) as Box<dyn Library>);
    libraries.add_library(Box::new(NavidromeLibrary::from_config(&config)) as Box<dyn Library>);
    time_library(Box::new(libraries));
}