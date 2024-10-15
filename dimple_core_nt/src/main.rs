use std::env;

use dimple_core_nt::{library::Library, scanner::Scanner};

fn main() {
    let args: Vec<String> = env::args().collect();
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
        println!("Library now contains {} tracks.", library.list_tracks().len());
    }
    else if command == "list" {
        let tracks = library.list_tracks();
        println!("{:30} | {:20} | {:40} | {:30} | {:50}", "Key", "Artist", "Album", "Title", "Path");
        for track in tracks {
            println!("{:30} | {:20} | {:40} | {:30} | {:50}", 
                track.key.unwrap_or_default(),
                track.artist.unwrap_or_default(),
                track.album.unwrap_or_default(), 
                track.title.unwrap_or_default(),
                track.path.unwrap_or_default());
        }
    }
}