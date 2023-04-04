use dimple::dimple::Settings;

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let config = config::Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()
        .unwrap();
    let _settings: Settings = config.try_deserialize().unwrap();

    // let mut source = None;
    // for config in settings.libraries {
    //     let library = match config {
    //         Navidrome(config) => Box::new(NavidromeLibrary::from_config(config)) as Box<dyn Library>,
    //         Local(config) => Box::new(LocalLibrary::from_config(config)) as Box<dyn Library>,
    //     };
    //     source = Some(library);
    //     break;
    // }

    // for release in source.releases() {
    //     println!("Merging {} {}", release.artists[0].name, release.title);
    //     // println!("Merging {:#?}", release);
    //     dest.merge_release(&source, &release).unwrap();
    // }
}
