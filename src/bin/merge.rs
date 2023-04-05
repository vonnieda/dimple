use dimple::{dimple::Settings, librarian::Librarian};

// TODO now that libraries is all sorted, add command line args here
//      print the list of libraries and then allow src and dest
fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let settings = Settings::default();
    let librarian = Librarian::from(settings.libraries);

    let cache = librarian.cache();
    for library in librarian.libraries().read().unwrap().iter() {
        for release in library.as_ref().as_ref().releases().iter() {
            let library = library.clone();
            let cache = cache.clone();
            log::info!("Merging {} {} -> {}",
                library.name(), release.title, cache.name());
            cache.merge_release(library.as_ref().as_ref(), &release).unwrap();
       }
    }
}
