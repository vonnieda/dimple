use std::path::Path;

use audiotags::Tag;
use dimple_folder_library::folder_library::FolderLibrary;
use walkdir::WalkDir;

/**
 * Keep finding bugs in all the lib, so getting closer and closer to the core.
 * Symphonia: Can't read the tags on 90% of the files due to:
 * audiotags: Blew up on a file that didn't have an extension due to unwrap().
 *            Fixed that, and it works basically as well.
 *            Okay actually now it reads dang near everything.
 * 
 */
fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();
    // builder.filter(Some("symphonia_core::probe"), log::LevelFilter::Off);
    // builder.filter(Some("symphonia_metadata::id3v2"), log::LevelFilter::Warn);
    // builder.filter(Some("symphonia_bundle_mp3::demuxer"), log::LevelFilter::Warn);
    // builder.filter(Some("symphonia_format_isomp4::demuxer"), log::LevelFilter::Warn);

    builder.init();
    
    // let folder_library = FolderLibrary::new();

    let path = Path::new("/Users/jason/Music/My Music");
    let files: Vec<walkdir::DirEntry> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some())
        .collect();
    for entry in files {
        let mut tag = Tag::new().read_from_path(entry.path());
        // let tag = Tag::read_from_path(entry.path());
        match tag {
            Ok(tag) => {
                println!("PASS {} {} {} {}", 
                    tag.artist().unwrap(),
                    tag.album_title().unwrap(),
                    tag.title().unwrap(),
                    entry.path().to_str().unwrap(), 
                );
            },
            Err(e) => {
                println!("FAIL {} {}", entry.path().to_str().unwrap(), e);
            }
        }
    }
}
