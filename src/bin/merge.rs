use dimple::music_library::{local::LocalLibrary, Library, navidrome::NavidromeLibrary};

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build().expect("Config error");

    let source = NavidromeLibrary::from_config(&config);
    let dest = LocalLibrary::new("data/library");

    for release in source.releases() {
        // println!("Merging {} {}", release.artists[0].name, release.title);
        println!("Merging {:#?}", release);
        dest.merge_release(&source, &release).unwrap();
        return;
    }
}
