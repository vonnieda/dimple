use std::{env, sync::Arc};

use dimple_core_nt::{library::Library, model::{Blob, ChangeLog, Track}, player::Player, scanner::Scanner, sync::{s3_storage::S3Storage, Sync}};

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();
    builder.init();

    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() {
        println!("Help:");
        println!("    import [/media/my_music]        Import tracks from the directory.");
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
    let library_path = "dimple.db";
    let library = Arc::new(Library::open(library_path));

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
        let directory = &args[2];
        println!("Library currently contains {} tracks.", library.tracks().len());
        println!("Scanning {}.", directory);
        let media_files = Scanner::scan_directory(directory);
        println!("Scanned {} media files.", media_files.len());

        println!("Importing {} media files.", media_files.len());
        library.import(&media_files);
        println!("Library now contains {} tracks.", library.tracks().len());
    }
    else if command == "tracks" {
        let tracks = library.tracks();
        for track in tracks {
            print_track(&track);
        }
    }
    else if command == "like" {
        let track_key = &args[2];
        let mut track: Track = library.get(track_key).unwrap();
        track.liked = !track.liked;
        library.save(&track);
        let track = library.get(track_key).unwrap();
        print_track(&track);
    }
    else if command == "queue" {
        let play_queue = player.play_queue();
        for track in play_queue.tracks {
            print_track(&track);
        }
    }
    else if command == "add" {
        let track_key = &args[2];
        player.play_queue_add(track_key);
        let play_queue = player.play_queue();
        for track in play_queue.tracks {
            print_track(&track);
        }
    }
    else if command == "clear" {
        player.play_queue_clear();
        let play_queue = player.play_queue();
        for track in play_queue.tracks {
            print_track(&track);
        }
    }
    else if command == "play" {
        player.play();
    }
    else if command == "sync" {
        library.sync();
    } 
    else if command == "changelogs" {
        let mut i = 0;
        for changelog in library.changelogs() {
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
}

fn print_track(track: &Track) {
    println!("{:30} | {:20} | {:40} | {:30} | {}", 
        track.key.clone().unwrap_or_default(),
        track.artist.clone().unwrap_or_default(),
        track.album.clone().unwrap_or_default(), 
        track.title.clone().unwrap_or_default(),
        track.liked);
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
