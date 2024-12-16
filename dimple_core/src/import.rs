use crate::{library::Library, model::{Blob, MediaFile, Track, TrackSource}};

pub mod media_file;
pub mod spotify;

use media_file::ScannedFile;
use symphonia::core::meta::StandardTagKey;
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    WalkDir::new(path).into_iter()
        .filter(|dir_entry| dir_entry.is_ok())
        .map(|dir_entry| dir_entry.unwrap())
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .map(|file_entry| ScannedFile::new(file_entry.path().to_str().unwrap()))
        .filter_map(|mf| mf.ok())
        .for_each(|x| import_internal(library, &x));
}


// /// Import MediaFiles into the Library, creating or updating Tracks,
// /// TrackSources, Blobs, etc.
// /// TODO okay this is slow cause we are scanning all the files first no
// /// matter what, reading all their tags and images and shit, and we might
// /// just ignore that file based on it's sha, so fix that.
fn import_internal(library: &Library, input: &ScannedFile) {
    // TODO txn
    let file_path = std::fs::canonicalize(&input.path).unwrap();
    let file_path = file_path.to_str().unwrap();

    let blob = Blob::read(file_path);
    let blob = library.find_blob_by_sha256(&blob.sha256)
        .or_else(|| Some(library.save(&blob)))
        .unwrap();

    let media_file = library.find_media_file_by_file_path(file_path)
        .or_else(|| Some(library.save(&MediaFile {
            file_path: file_path.to_owned(),
            sha256: blob.sha256.clone(),
            artist: input.tag(StandardTagKey::Artist),
            album: input.tag(StandardTagKey::Album),
            title: input.tag(StandardTagKey::TrackTitle),
            ..Default::default()
        })))
        .unwrap();

    if library.track_sources_by_blob(&blob).is_empty() {
        // TODO temp, eventually uses more matching
        // or maybe just always create and de-dupe?
        let track = library.find_track_for_media_file(&media_file)
            .or_else(|| Some(library.save(&Track {
                artist: media_file.artist,
                album: media_file.album,
                title: media_file.title,
                ..Default::default()
            })))
            .unwrap();

        let _source = library.save(&TrackSource {
            track_key: track.key.unwrap(),
            blob_key: blob.key.unwrap(),
            ..Default::default()
        });
    }
}
