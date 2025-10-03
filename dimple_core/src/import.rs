pub mod spotify;
pub mod lofty_tagged_media_file;
pub mod symphonia_tagged_media_file;

use std::path::Path;

use crate::{import::symphonia_tagged_media_file::SymphoniaTaggedMediaFile, librarian::{self, ReleaseGroupMetadata, ReleaseMetadata, TrackMetadata}, library::Library, model::{MediaFile, ModelBasics as _, Track, TrackSource}};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use itertools::Itertools as _;
use lofty_tagged_media_file::LoftyTaggedMediaFile;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    let force = true;
    
    log::info!("Importing {path}.");

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    files.par_iter().for_each(|file| {
        let path = Path::new(&file.path);
        if let Err(e) = import_single_file(library, path, force) {
            log::error!("  Error reading {path:?}: {e}");
        }
    });
}

fn scan(path: &str) -> Vec<ScannedFile> {
    // TODO was trying to defer to Symphonia, but I think it's reasonable to
    // know which files are supported based on the playback library. So, change
    // to INCLUDE_EXTENSIONS, basically. 
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
    log::debug!("Importing {path:?}.");

    // Read the tags from the file.
    let tags = LoftyTaggedMediaFile::new(path)?;
    let metadata = tags.metadata();
    dbg!(&metadata);
    let mut track_metadata = metadata.track;
    if track_metadata.track.length_ms.is_none() {
        log::warn!("  No track length found {}, attempting to calculate", path.to_string_lossy());
        let symph = SymphoniaTaggedMediaFile::new(path)?;
        if let Some(length) = symph.track_metadata().track.length_ms {
            track_metadata.track.length_ms = Some(length);
        }
        else {
           return Err(anyhow!("Unable to find or calculate track length {}", path.to_string_lossy()))
        }
    }

    let file_path = path.to_str().unwrap();
    // TODO Don't wanna read the whole file, just check modified unless forced.
    let file_content = Some(std::fs::read(path)?);
    let track_source = library.db.transaction(|txn| {
        // Create or update a MediaFile by the file path.
        let mut media_file: MediaFile = txn
            .find("SELECT * FROM MediaFile WHERE file_path = ?", (file_path,))?
            .unwrap_or_default();
        media_file.file_path = path.to_str().unwrap().to_string();
        media_file.last_imported = Utc::now();
        media_file.last_modified = path.metadata()?.modified()?.into();
        // TODO temporary, waiting on blob support
        media_file.content = file_content; 
        let media_file = txn.save(&media_file)?;

        // Find or create a TrackSource by the MediaFile id. This is not yet saved,
        // since it will be updated below.
        let mut track_source: TrackSource = txn
            .find("SELECT * FROM TrackSource WHERE media_file_id = ?",  (&media_file.id,))?
            .unwrap_or_default();

        // TODO need to complete handling of missing release group, release,
        // artists, and title. 

        // Resolve and merge the ReleaseGroup from the metadata.
        // If we have an artist in the metadata, use merge_artist_release_group to scope
        // the search to that artist. Otherwise, use the generic merge_release_group.
        let release_group = if let Some(artist_metadata) = metadata.release_group.artists.first() {
            let artist = librarian::merge_artist(txn, &artist_metadata.artist)?;
            librarian::merge_artist_release_group(txn, &artist, &metadata.release_group.release_group)?
        } else {
            librarian::merge_release_group(txn, &metadata.release_group.release_group)?
        };
        let release_group = librarian::merge_release_group_metadata(txn, &release_group, &metadata.release_group)?;

        // Resolve and merge the Release, scoped to the ReleaseGroup.
        let release = librarian::merge_release_group_release(txn, &release_group, &metadata.release.release)?;
        let release = librarian::merge_release_metadata(txn, &release, &metadata.release)?;

        // Resolve and merge the Track, scoped to the Release.
        let track = librarian::merge_release_track(txn, &release, &track_metadata.track)?;
        let track = librarian::merge_track_metadata(txn, &track, &track_metadata)?;

        // Update the TrackSource with the saved track_id and media_file_id.
        track_source.track_id = track.id.clone();
        track_source.media_file_id = media_file.id.clone();
        let track_source = txn.save(&track_source)?;

        Ok(track_source)
    }).unwrap();

    log::info!("Imported {} {}: {}", 
        track_metadata.track.title.unwrap_or("(Unknown Title)".to_string()),
        // track_metadata.release.unwrap().release.title.unwrap_or("(Unknown Release)".to_string()),
        track_metadata.artists.iter().map(|f| f.artist.name.clone().unwrap_or("(Unknown Artist)".to_string()).to_string()).join(","),
        path.file_name().unwrap().to_str().unwrap(),
    );
    Ok(track_source)
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct MediaFileMetadata {
    pub release_group: ReleaseGroupMetadata,
    pub release: ReleaseMetadata,
    pub track: TrackMetadata,
}

#[derive(Debug)]
struct ScannedFile {
    path: String,
    last_modified: DateTime<Utc>,
    file_length: u64,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::MediaFile};

    #[test]
    fn import() {
        let library = Library::open_memory();
        assert_eq!(library.list::<MediaFile>().len(), 0);
        library.import("tests/data/media_files");
        let num_mediafiles = library.list::<MediaFile>().len();
        assert_eq!(library.list::<MediaFile>().len(), 3);
        library.import("tests/data/media_files");
        assert_eq!(library.list::<MediaFile>().len(), num_mediafiles);
    }    
}

