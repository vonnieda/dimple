pub mod spotify;
pub mod tagged_media_file;

use std::path::Path;

use crate::{librarian, library::Library, merge::CrdtRules, model::{Artist, Dimage, DimageRef, Genre, Link, MediaFile, ModelBasics as _, Release, Track, TrackSource}};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use symphonia::core::meta::StandardVisualKey;
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
    media_file.last_imported = Utc::now();
    media_file.last_modified = path.metadata()?.modified()?.into();
    let media_file = media_file.save(library);
    log::info!("MediaFile {:?}", media_file.key);
    
    // Find or create a TrackSource by the MediaFile key. This is not yet saved,
    // since it will be updated below.
    let mut track_source = TrackSource::find(library, 
        "SELECT * FROM TrackSource WHERE media_file_key = ?", 
        (&media_file.key,)).unwrap_or_default();
    
    // Match and merge the Track, preferring the one on the TrackSource if it
    // exists.
    let metadata = tags.track_metadata();
    println!("{:?}", metadata.artists.iter().map(|a| a.artist.name.clone()).collect::<Vec<_>>());
    // for tag in &tags.tags {
    //     println!("{:?} {} {}", tag.std_key, tag.key, tag.value.to_string());
    // }
    // dbg!(&track_metadata);

    return Err(anyhow!("asd"));

    let track = track_source.track(library).unwrap_or_default();
    let track = CrdtRules::merge(track, tags.track());
    if track.title.is_none() {
        log::warn!("No track title {}", path.to_string_lossy());
    }
    // let track = librarian::merge_track(library, &track);

    // Update the TrackSource with the saved track_key.
    track_source.track_key = track.key.clone();
    track_source.media_file_key = media_file.key.clone();
    let track_source = track_source.save(library);
    log::info!("TrackSource {:?}", track_source.key);

    // Import images
    for visual in tags.visuals.iter() {
        if let Ok(dymage) = image::load_from_memory(&visual.data) {
            // TODO this should be a match/merge
            let dimage = Dimage::new(&dymage).save(library);
            let _ = library.save(&DimageRef {
                model_key: track.release_key.clone().unwrap(),
                dimage_key: dimage.key.clone().unwrap(),
                ..Default::default()
            });
        }
    }

    // log::info!("Imported    {:30} {:30} {:60} {}", 
    //     track.artist_name(library).unwrap_or_default(), 
    //     track.album_name(library).unwrap_or_default(),
    //     track.title.unwrap_or_default(),
    //     path.file_name().unwrap_or_default().to_str().unwrap_or_default());

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

