pub mod spotify;
pub mod lofty_tagged_media_file;
pub mod symphonia_tagged_media_file;

use std::path::Path;

use crate::{import::symphonia_tagged_media_file::SymphoniaTaggedMediaFile, librarian, library::Library, model::{MediaFile, ModelBasics as _, Track, TrackSource}};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use dimple_db::db::transaction::DbTransaction;
use lofty_tagged_media_file::LoftyTaggedMediaFile;
use walkdir::WalkDir;

pub fn import(library: &Library, path: &str) {
    let force = true;
    
    log::info!("Importing {}.", path);

    let files = scan(path);
    log::info!("Scanned {} files.", files.len());

    files.iter().for_each(|file| {
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

/// Time to actually think through what happens in an import:
/// - Create a Db transaction to perform all the matching and updating.
/// - Do any slow work up front, before creating a Db transaction.
/// - MediaFile has a file path, which is of course device specific. Part of
/// that issue is that we sync all tables, which means we sync MediaFiles
/// with paths that don't make sense on every device. 
///     - This is not super critical right now. Other devices may try to look
///     up files not on this device - no big deal.
/// - We need to be able to sync the music, but it's a bummer that we end up
/// with a copy at the original file, a copy in the changelog, and a copy
/// in the database.
///     - Specialized blob handling is one option. I keep thinking of a method
///     where we can refer to the blob in the changelog and not keep a copy in
///     the entities. Pull it in on the fly.
///     - This one is not critical right now. Storage is cheapish.
fn import_single_file(library: &Library, path: &Path, _force: bool) -> Result<TrackSource, anyhow::Error> {
    if !path.is_file() {
        return Err(anyhow::anyhow!("Path must be a file: {:?}", path));
    }
    log::info!("Importing {:?}.", path);

    // Read the tags from the file.
    let tags = LoftyTaggedMediaFile::new(path)?;
    let mut track_metadata = tags.track_metadata();
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
    if track_metadata.track.length_ms.is_none() {
        log::warn!("  No track length tag {}, attempting to calculate", path.to_string_lossy());
        let symph = SymphoniaTaggedMediaFile::new(path)?;
        if let Some(length) = symph.track_metadata().track.length_ms {
            track_metadata.track.length_ms = Some(length);
        }
        else {
           return Err(anyhow!("Unable to find or calculate track length {}", path.to_string_lossy()))
        }
    }
    log::debug!("{:?} {:?} {:?} {:?}", 
        path.file_name().unwrap(), 
        track_metadata.clone().artists.get(0).map(|f| f.artist.name.clone().unwrap_or_default().to_string()),
        track_metadata.clone().release.unwrap().release.title,
        track_metadata.clone().track.title);

    let file_path = path.to_str().unwrap();
    let file_content = Some(std::fs::read(path)?);
    let track_source = library.db.transaction(move |txn| {
        // Create or update a MediaFile by the file path.
        let mut media_file: MediaFile = txn
            .find("SELECT * FROM MediaFile WHERE file_path = ?", (file_path,))?
            .unwrap_or_default();
        media_file.file_path = path.to_str().unwrap().to_string();
        media_file.last_imported = Utc::now();
        media_file.last_modified = path.metadata()?.modified()?.into();
        // TODO temporary, waiting on blob support
        // TODO STOPSHIP move read outside of txn 
        media_file.content = file_content; 
        let media_file = txn.save(&media_file)?;

        // Find or create a TrackSource by the MediaFile id. This is not yet saved,
        // since it will be updated below.
        let mut track_source: TrackSource = txn
            .find("SELECT * FROM TrackSource WHERE media_file_id = ?",  (&media_file.id,))?
            .unwrap_or_default();
        
        // Match and merge the Track, preferring the one on the TrackSource if it
        // exists.
        let pre_match: Option<Track> = track_source.track_id.clone().and_then(|id| txn.get(&id).ok()).flatten();
        let track = librarian::merge_track_metadata(txn, &track_metadata, pre_match)?;        
        // Update the TrackSource with the saved track_id.
        track_source.track_id = track.id.clone();
        track_source.media_file_id = media_file.id.clone();
        let track_source = txn.save(&track_source)?;        
        Ok(track_source)
    }).unwrap();

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

