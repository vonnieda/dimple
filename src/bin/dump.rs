use dimple::{dimple::Settings, librarian::Librarian, music_library::Library};

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let settings = Settings::default();
    let librarian = Librarian::from(settings.libraries);

    for release in librarian.releases().iter() {
        log::info!("{} {} {}", 
            release.artists.first().unwrap().name,
            release.title,
            release.url);
    }
}
