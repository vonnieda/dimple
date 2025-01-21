pub mod spotify;
pub mod tagged_media_file;

use std::path::Path;

use crate::{librarian, library::Library, merge::CrdtRules, model::{Dimage, DimageRef, MediaFile, ModelBasics as _, Track, TrackSource}};

use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use tagged_media_file::TaggedMediaFile;
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    let force = true;
    
    log::info!("Importing {}.", path);

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    files.par_iter().for_each(|file| {
        let _ = import_single_file(&library, Path::new(&file.path), force);
    });
}

fn scan(path: &str) -> Vec<ScannedFile> {
    let files = WalkDir::new(path).into_iter()
        .filter_map(|dir_entry| dir_entry.ok())
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .filter(|dir_entry| dir_entry.file_name() != ".DS_Store")
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
    
    // Read the tags from the file.
    let tags = TaggedMediaFile::new(path)?;

    // Create or update a MediaFile by the file path.
    let mut media_file = library.find_media_file_by_file_path(path.to_str().unwrap())
        .unwrap_or_default();
    media_file.file_path = path.to_str().unwrap().to_string();
    media_file.last_imported = Utc::now();
    media_file.last_modified = path.metadata()?.modified()?.into();
    let media_file = media_file.save(library);
    
    // Find or create a TrackSource by the MediaFile key.
    let mut track_source: TrackSource = library.find("
        SELECT * FROM TrackSource WHERE media_file_key = ?
        ", (&media_file.key,))
        .unwrap_or_default();
    
    // Match and merge the Track, preferring the one on the TrackSource if it
    // exists.
    let track = track_source.track_key.as_ref()
        .and_then(|track_key| Track::get(library, &track_key))
        .or_else(|| librarian::match_track(library, &tags))
        .unwrap_or_default();
    let track = CrdtRules::merge(track, tags.track());
    if track.title.is_none() {
        log::warn!("No track title {}", path.to_string_lossy());
    }
    let mut track = librarian::merge_track(library, &track, &tags);

    // Match and merge the Release, preferring the one on the Track if it
    // exists.
    let release = track.release(library)
        .or_else(|| librarian::match_release(library, &tags))
        .unwrap_or_default();
    let release = CrdtRules::merge(release, tags.release());
    if release.title.is_none() {
        log::warn!("No release title {}", path.to_string_lossy());
    }
    let release = librarian::merge_release(library, &release, &tags);

    // Update the track with the (maybe newly created) release_key and save it.
    if track.release_key != release.key {
        track.release_key = release.key.clone();
        track = track.save(library);
    }

    // Update the TrackSource with the saved track_key.
    track_source.track_key = track.key.clone();
    track_source.media_file_key = media_file.key.clone();
    let track_source = track_source.save(library);

    // Import images
    for visual in tags.visuals.iter() {
        if let Ok(dymage) = image::load_from_memory(&visual.data) {
            let dimage = Dimage::new(&dymage).save(library);
            let _ = library.save(&DimageRef {
                model_key: release.key.clone().unwrap(),
                dimage_key: dimage.key.clone().unwrap(),
                ..Default::default()
            });
            log::info!("  Saved image for release {:?} {}", release.title, dimage.sha256);
        }
    }

    log::info!("Imported    {:30} {:30} {:60} {}", 
        track.artist_name(library).unwrap_or_default(), 
        track.album_name(library).unwrap_or_default(),
        track.title.unwrap_or_default(),
        path.file_name().unwrap_or_default().to_str().unwrap_or_default());

    Ok(track_source)
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
