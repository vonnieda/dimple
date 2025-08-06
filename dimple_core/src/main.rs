use std::{env, path::Path, sync::Arc, time::Duration};

use dimple_core::{import::spotify, library::Library, model::{Artist, DimpleEntity, ModelBasics as _, Release, Track}, player::Player};
use directories::ProjectDirs;

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();

    // [2024-01-24T21:03:58.412Z INFO  symphonia_core::probe] found the format marker [49, 44, 33] @ 0+2 bytes.
    builder.filter(Some("symphonia_core"), log::LevelFilter::Off);

    // [2024-01-24T21:02:27.904Z INFO  symphonia_bundle_mp3::demuxer] estimating duration from bitrate, may be inaccurate for vbr files
    builder.filter(Some("symphonia_bundle_mp3"), log::LevelFilter::Off);

    // [2024-01-24T21:02:27.905Z INFO  symphonia_metadata::id3v2] unsupported frame UFID
    builder.filter(Some("symphonia_metadata"), log::LevelFilter::Off);

    // [2024-01-24T21:06:24.917Z INFO  symphonia_format_isomp4::demuxer] stream is seekable with len=3037538 bytes.
    builder.filter(Some("symphonia_format_isomp4"), log::LevelFilter::Off);

    // [2025-03-20T14:35:24.952Z WARN  tiny_skia::painter] empty paths and horizontal/vertical lines cannot be filled
    builder.filter(Some("tiny_skia::painter"), log::LevelFilter::Off);

    builder.init();


    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() {
        println!("Help:");
        println!("    import [/media/my_music]        Import tracks from the file or directory.");
        println!("    tracks                          List all tracks in the library.");
        println!("    like [1234-12341234-1234-1234]  Toggle 'liked' for the specified track key.");
        println!("    queue                           List the tracks in the play queue.");
        println!("    add [1234-12341234-1234-1234]   Add the track to the queue using the track key from the tracks command.");
        println!("    clear                           Clear the play queue.");
        println!("    play                            Play the play queue from start to finish.");
        println!("    sync                            Sync the library with an S3 target.");
        return
    }


    let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
    let mut data_dir = dirs.data_dir().to_path_buf();
    let mut config_dir = dirs.config_dir().to_path_buf();
    let mut cache_dir = dirs.cache_dir().to_path_buf();
    if let Some(root) = env::var("DIMPLE_ROOT").ok() {
        let root_dir = Path::new(&root.to_string()).to_path_buf();
        data_dir = root_dir.join("data").to_path_buf();
        config_dir = root_dir.join("config").to_path_buf();
        cache_dir = root_dir.join("cache").to_path_buf();
    }
    let library_path = data_dir.join("library.db");
    let config_path = config_dir.join("config.db");
    let image_cache_dir = cache_dir.join("image_cache");
    dbg!(&data_dir, &cache_dir, &library_path, &image_cache_dir, &config_path);
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&image_cache_dir).unwrap();

    let library = Arc::new(Library::open(library_path.to_str().unwrap()));

    let player = Player::new(library.clone());
    let command = &args[1];
    if command == "import" {
        let path = &args[2];
        println!("Library currently contains {} tracks.", Track::list(&library).len());
        println!("Importing {}.", path);
        library.import(&path);
        println!("Library now contains {} tracks, {} releases, {} artists.", 
            Track::list(&library).len(),
            Release::list(&library).len(),
            Artist::list(&library).len());
    }
    else if command == "artists" {
        for artist in Artist::list(&library).iter() {
            print_artist(&library, &artist);
        }
    }
    else if command == "releases" {
        for release in Release::list(&library).iter() {
            print_release(&library, &release);
        }
    }
    else if command == "tracks" {
        let tracks = Track::list(&library);
        for track in tracks {
            print_track(&library, &track);
        }
    }
    else if command == "queue" {
        let play_queue = player.queue();
        for track in play_queue.tracks(&library) {
            print_track(&library, &track);
        }
    }
    else if command == "add" {
        let track_key = &args[2];
        let track = Track::get(&library, &track_key).unwrap();
        player.queue().append(&library, &track.into());
        for track in player.queue().tracks(&library) {
            print_track(&library, &track);
        }
    }
    else if command == "clear" {
        player.queue().clear(&library);
        let play_queue = player.queue();
        for track in play_queue.tracks(&library) {
            print_track(&library, &track);
        }
    }
    else if command == "play" {
        player.play();
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    else if command == "sync" {
        library.sync();
    } 
    if command == "import_spotify" {
        let path = &args[2];
        spotify::import(&library, path);
    }
}

fn print_artist(library: &Library, artist: &Artist) {
    println!("{:30}", artist.name.clone().unwrap_or_default());
}

fn print_release(library: &Library, release: &Release) {
    println!("{:30} | {:20} | {:40}", 
        release.id.clone().unwrap_or_default(),
        release.title.clone().unwrap_or_default(),
        release.artist_name(library).unwrap_or_default());
}

fn print_track(library: &Library, track: &Track) {
    println!("{:30} | {:20} | {:40} | {:30}", 
        track.id.clone().unwrap_or_default(),
        track.artist_name(library).unwrap_or_default(),
        track.album_name(library).unwrap_or_default(), 
        track.title.clone().unwrap_or_default());
}
