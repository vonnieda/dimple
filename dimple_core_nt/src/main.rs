use std::env;

use dimple_core_nt::{library::Library, model::Track, scanner::Scanner};

use playback_rs::{Player, Song};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() {
        println!("Help:");
        println!("    import [/media/my_music]        Import tracks from the directory.");
        println!("    tracks                          List all tracks in the library.");
        println!("    queue                           List the tracks in the play queue.");
        println!("    add 1234-12341234-1234-1234     Add the track to the queue using the track key from the tracks command.");
        return
    }
    let library_path = "dimple.db";
    let library = Library::open(library_path);
    println!("Opened library {}.", library_path);
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
        let play_queue = library.play_queue();
        for track in play_queue.tracks {
            print_track(&track);
        }
    }
    else if command == "add" {
        let track_key = &args[2];
        library.play_queue_add(track_key);
        let play_queue = library.play_queue();
        for track in play_queue.tracks {
            print_track(&track);
        }
    }
    else if command == "play" {
        let player = Player::new(None).unwrap();
        let play_queue = library.play_queue();
        let filenames = play_queue.tracks.iter().map(|track| track.path.clone().unwrap());
        for next_song in filenames {
            println!("Loading song '{}'...", next_song);
            let song = Song::from_file(&next_song, None).unwrap();
            println!("Waiting for queue space to become available...");
            while player.has_next_song() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            println!(
                "Queueing next song '{}' with {:?} left in current song...",
                next_song,
                player.get_playback_position()
            );
            player.play_song_next(&song, None).unwrap();
            println!("Queued.");
        }
        println!("Waiting for songs to finish.");
        while player.has_current_song() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!("Exiting.");    
    }
}

fn print_track(track: &Track) {
    println!("{:30} | {:20} | {:40} | {:30} | {:50}", 
        track.key.clone().unwrap_or_default(),
        track.artist.clone().unwrap_or_default(),
        track.album.clone().unwrap_or_default(), 
        track.title.clone().unwrap_or_default(),
        track.path.clone().unwrap_or_default());
}