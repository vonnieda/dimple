use std::{env, sync::Arc};

use dimple_core_nt::{library::Library, model::Track, player::Player, scanner::Scanner};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() {
        println!("Help:");
        println!("    import [/media/my_music]        Import tracks from the directory.");
        println!("    tracks                          List all tracks in the library.");
        println!("    queue                           List the tracks in the play queue.");
        println!("    add 1234-12341234-1234-1234     Add the track to the queue using the track key from the tracks command.");
        println!("    clear                           Clear the play queue.");
        println!("    play                            Play the play queue from start to finish.");
        return
    }
    let library_path = "dimple.db";
    let library = Arc::new(Library::open(library_path));
    let player = Player::new(library.clone());
    let command = &args[1];
    if command == "import" {
        let directory = &args[2];
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
}

fn print_track(track: &Track) {
    println!("{:30} | {:20} | {:40} | {:30}", 
        track.key.clone().unwrap_or_default(),
        track.artist.clone().unwrap_or_default(),
        track.album.clone().unwrap_or_default(), 
        track.title.clone().unwrap_or_default());
}