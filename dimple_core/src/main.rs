use std::{env, sync::Arc, time::Duration};

use dimple_core::{import::spotify, library::Library, model::{Artist, Blob, ChangeLog, ModelBasics as _, Release, Track}, player::Player, sync::{s3_storage::S3Storage, Sync}};
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
        println!("    changelogs                      List changelogs.");
        println!("    blobs                           List blobs.");
        return
    }

    let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
    let data_dir = dirs.data_dir();
    let cache_dir = dirs.cache_dir();
    let config_dir = dirs.config_dir();
    let image_cache_dir = cache_dir.join("image_cache");
    let library_path = data_dir.join("library.db");
    dbg!(&data_dir, &cache_dir, &config_dir, &library_path, &image_cache_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&image_cache_dir).unwrap();

    let library = if let Some(path) = env::var("DIMPLE_LIBRARY_PATH").ok() {
        Arc::new(Library::open(&path))
    }
    else {
        Arc::new(Library::open(library_path.to_str().unwrap()))    
    };

    let access_key = env::var("DIMPLE_TEST_S3_ACCESS_KEY").unwrap();
    let secret_key = env::var("DIMPLE_TEST_S3_SECRET_KEY").unwrap();
    let region = env::var("DIMPLE_TEST_S3_REGION").unwrap();
    let endpoint = env::var("DIMPLE_TEST_S3_ENDPOINT").unwrap();
    let bucket = env::var("DIMPLE_TEST_S3_BUCKET").unwrap();
    let prefix = env::var("DIMPLE_TEST_S3_PREFIX").unwrap();
    let storage = S3Storage::new(&access_key, &secret_key, &region, &endpoint, &bucket, &prefix);
    // let storage = MemoryStorage::default();
    let sync = Sync::new(Box::new(storage), &prefix);
    library.add_sync(sync);

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
        player.queue().append(&library, &Track::get(&library, &track_key).unwrap());
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
    else if command == "changelogs" {
        let mut i = 0;
        for changelog in ChangeLog::list(&library) {
            print_changelog(&changelog);
            i += 1;
        }
        println!("{} changelogs", i);
    }
    else if command == "blobs" {
        let mut i = 0;
        for blob in library.list::<Blob>() {
            println!("{:?}", blob);
            i += 1;
        }
        println!("{} blobs", i);
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
        release.key.clone().unwrap_or_default(),
        release.title.clone().unwrap_or_default(),
        release.artist_name(library).unwrap_or_default());
}

fn print_track(library: &Library, track: &Track) {
    println!("{:30} | {:20} | {:40} | {:30}", 
        track.key.clone().unwrap_or_default(),
        track.artist_name(library).unwrap_or_default(),
        track.album_name(library).unwrap_or_default(), 
        track.title.clone().unwrap_or_default());
}

fn print_changelog(changelog: &ChangeLog) {
    println!("{:16} | {:16} | {:16} | {:16} | {:16} | {:16} | {:16}", 
        changelog.timestamp.clone(),
        changelog.actor.clone(), 
        changelog.model.clone(),
        changelog.op.clone(),
        changelog.model_key.clone(),
        changelog.field.clone().unwrap_or_default(),
        changelog.value.clone().unwrap_or_default());
}
