pub mod spotify;
pub mod lofty_tagged_media_file;
pub mod symphonia_tagged_media_file;

use std::path::Path;

use crate::{librarian, library::Library, merge::CrdtRules, model::{Artist, Dimage, DimageRef, Genre, Link, MediaFile, ModelBasics as _, Release, Track, TrackSource}};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use lofty_tagged_media_file::LoftyTaggedMediaFile;
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    let force = true;
    
    log::info!("Importing {}.", path);

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    files.par_iter().for_each(|file| {
        let path = Path::new(&file.path);
        if let Err(e) = import_single_file(&library, path, force) {
            log::error!("  Error reading {:?}: {}", path, e);
        }
    });
}

fn scan(path: &str) -> Vec<ScannedFile> {
    const IGNORE_EXTENSIONS: [&str;5] = ["jpg", "png", "pdf", "m4p", "DS_Store"];
    const IGNORE_FILENAMES: [&str;1] = [".DS_Store"];

    let files = WalkDir::new(path).into_iter()
        .filter_map(|dir_entry| dir_entry.ok())
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .filter(|dir_entry| !IGNORE_FILENAMES.contains(&dir_entry.file_name().to_str().unwrap()))
        .filter(|dir_entry| !IGNORE_EXTENSIONS.contains(&dir_entry.path().extension().unwrap_or_default().to_ascii_lowercase().to_str().unwrap()))
        .map(|dir_entry| ScannedFile {
            path: dir_entry.path().to_str().unwrap().to_string(),
            last_modified: dir_entry.metadata().unwrap().modified().unwrap().into(),
            file_length: dir_entry.metadata().unwrap().len(),
        })
        .collect::<Vec<_>>();
    files
}

fn import_single_file(library: &Library, path: &Path, _force: bool) -> Result<TrackSource, anyhow::Error> {
    if !path.is_file() {
        return Err(anyhow::anyhow!("Path must be a file: {:?}", path));
    }
    // log::info!("Importing {:?}.", path);

    // Read the tags from the file.
    let tags = LoftyTaggedMediaFile::new(path)?;
    let track_metadata = tags.track_metadata();
    if track_metadata.track.title.is_none() {
        log::warn!("  No track title {}", path.to_string_lossy());
    }
    if track_metadata.release.is_none() {
        log::warn!("  No release {}", path.to_string_lossy());
    }
    if track_metadata.release.clone().unwrap().release.title.is_none() {
        log::warn!("  No release title {}", path.to_string_lossy());
    }
    if track_metadata.artists.is_empty() {
        log::warn!("  No artists {}", path.to_string_lossy());
    }
    // log::info!("{:?} {:?} {:?} {:?}", 
    //     path.file_name().unwrap(), 
    //     track_metadata.clone().artists.get(0).map(|f| f.artist.name.clone().unwrap_or_default().to_string()),
    //     track_metadata.clone().release.unwrap().release.title,
    //     track_metadata.clone().track.title);
    
    // Create or update a MediaFile by the file path.
    let mut media_file = library.find_media_file_by_file_path(path.to_str().unwrap())
        .unwrap_or_default();
    media_file.file_path = path.to_str().unwrap().to_string();
    media_file.last_imported = Utc::now();
    media_file.last_modified = path.metadata()?.modified()?.into();
    let media_file = media_file.save(library);
    
    // Find or create a TrackSource by the MediaFile key. This is not yet saved,
    // since it will be updated below.
    let mut track_source = TrackSource::find(library, 
        "SELECT * FROM TrackSource WHERE media_file_key = ?", 
        (&media_file.key,)).unwrap_or_default();
    
    // Match and merge the Track, preferring the one on the TrackSource if it
    // exists.
    let track = librarian::merge_track_metadata(library, &track_metadata, track_source.track(library));

    // Update the TrackSource with the saved track_key.
    track_source.track_key = track.key.clone();
    track_source.media_file_key = media_file.key.clone();
    let track_source = track_source.save(library);

    Ok(track_source)
}

fn print_track(track: &Track, library: &Library) {
    println!("{:?}", track.title);
    println!("  Artists: {:?}", track.artists(library).iter().map(|a| a.name.clone()).collect::<Vec<_>>());
    println!("  Genres: {:?}", track.genres(library).iter().map(|a| a.name.clone()).collect::<Vec<_>>());
    println!("  Release: {:?}", track.release(library).map(|r| r.key.clone()));
    println!("  Links: {:?}", track.links(library));
}

#[derive(Debug)]
struct ScannedFile {
    path: String,
    last_modified: DateTime<Utc>,
    file_length: u64,
}

mod tests {
    use crate::{library::Library, model::MediaFile};

    #[test]
    fn import() {
        let library = Library::open_memory();
        assert!(library.list::<MediaFile>().len() == 0);
        library.import("tests/data/media_files");
        let num_mediafiles = library.list::<MediaFile>().len();
        assert!(library.list::<MediaFile>().len() > 0);
        library.import("tests/data/media_files");
        assert!(library.list::<MediaFile>().len() == num_mediafiles);
    }    
}

