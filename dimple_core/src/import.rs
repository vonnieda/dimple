pub mod spotify;
pub mod tagged_media_file;

use std::path::Path;

use crate::{library::Library, merge::CrdtRules, model::{Artist, ArtistRef, Dimage, DimageRef, Genre, GenreRef, MediaFile, ModelBasics as _, Release, Track, TrackSource}};

use chrono::{DateTime, Utc};
use image::DynamicImage;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use sha2::Digest;
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
        .or_else(|| match_track(library, &tags))
        .unwrap_or_default();
    let track = CrdtRules::merge(track, tags.track());
    if track.title.is_none() {
        log::warn!("No track title {}", path.to_string_lossy());
    }
    let mut track = merge_track(library, &track, &tags);

    // Match and merge the Release, preferring the one on the Track if it
    // exists.
    let release = track.release(library)
        .or_else(|| match_release(library, &tags))
        .unwrap_or_default();
    let release = CrdtRules::merge(release, tags.release());
    if release.title.is_none() {
        log::warn!("No release title {}", path.to_string_lossy());
    }
    let release = merge_release(library, &release, &tags);

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

fn match_artist(library: &Library, artist: &Artist) -> Option<Artist> {
    library.find("
        SELECT Artist.* 
        FROM Artist 
        WHERE (musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1)
        OR (Artist.name IS NOT NULL AND Artist.name = ?2)
        ", (&artist.musicbrainz_id, &artist.name))
}

fn match_genre(library: &Library, genre: &Genre) -> Option<Genre> {
    library.find("
        SELECT Genre.* 
        FROM Genre 
        WHERE (musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1)
        OR (Genre.name IS NOT NULL AND Genre.name = ?2)
        ", (&genre.musicbrainz_id, &genre.name))
}

fn match_track(library: &Library, tags: &TaggedMediaFile) -> Option<Track> {
    let track = tags.track();
    let matched_track = library.find("
        SELECT Track.* 
        FROM Track 
        WHERE musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1", 
        (&track.musicbrainz_id,));
    if matched_track.is_some() {
        return matched_track
    }
    let release = tags.release();
    for artist in tags.track_artists() {
        let matched_track: Option<Track> = library.find("
            SELECT t.* FROM Track t
            LEFT JOIN Release r ON (r.key = t.release_key)
            LEFT JOIN ArtistRef tar ON (tar.model_key = t.key)
            LEFT JOIN Artist ta ON (ta.key = tar.artist_key)
            LEFT JOIN ArtistRef rar ON (rar.model_key = r.key)
            LEFT JOIN Artist ra ON (ra.key = rar.artist_key)
            WHERE (t.title = ?1 AND r.title = ?2 AND (ta.name = ?3 OR ra.name = ?3))
            ", (&track.title, &release.title, artist.name));
        if matched_track.is_some() {
            return matched_track
        }
    }
    None
}

// TODO matching hash of embedded artwork might be another good datapoint
fn match_release(library: &Library, tags: &TaggedMediaFile) -> Option<Release> {
    let release = tags.release();
    let matched_release = library.find("
        SELECT Release.* 
        FROM Release 
        WHERE musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1", 
        (&release.musicbrainz_id,));
    if matched_release.is_some() {
        return matched_release
    }
    for artist in tags.release_artists() {
        let matched_release: Option<Release> = library.find("
            SELECT r.* FROM Release r
            LEFT JOIN ArtistRef rar ON (rar.model_key = r.key)
            LEFT JOIN Artist ra ON (ra.key = rar.artist_key)
            WHERE (r.title = ?1 AND ra.name = ?2)
            ", (&release.title, artist.name));
        if matched_release.is_some() {
            return matched_release
        }
    }
    None
}

fn merge_track(library: &Library, track: &Track, tags: &TaggedMediaFile) -> Track {
    let track = track.save(library);

    // Match, update, and link Track Artists.
    for artist in tags.track_artists() {
        let matched = match_artist(library, &artist)
            .unwrap_or_default();
        let artist = CrdtRules::merge(matched, artist);
        let artist = artist.save(library);
        library.save(&ArtistRef {
            artist_key: artist.key.clone().unwrap(),
            model_key: track.key.clone().unwrap(),
            ..Default::default()
        });
    }

    // Match, update, and link Track Genres.
    for genre in tags.track_genres() {
        let matched = match_genre(library, &genre)
            .unwrap_or_default();
        let genre = CrdtRules::merge(matched, genre);
        let genre = genre.save(library);
        library.save(&GenreRef {
            genre_key: genre.key.clone().unwrap(),
            model_key: track.key.clone().unwrap(),
            ..Default::default()
        });
    }

    track
}

// fn match_dimage() ??

fn merge_release(library: &Library, release: &Release, tags: &TaggedMediaFile) -> Release {
    let release = release.save(library);

    // Match and update Release Artists.
    for artist in tags.release_artists() {
        let matched = match_artist(library, &artist)
            .unwrap_or_default();
        let artist = CrdtRules::merge(matched, artist);
        let artist = artist.save(library);
        library.save(&ArtistRef {
            artist_key: artist.key.clone().unwrap(),
            model_key: release.key.clone().unwrap(),
            ..Default::default()
        });
    }

    // Match, update, and link Release Genres.
    for genre in tags.release_genres() {
        let matched = match_genre(library, &genre)
            .unwrap_or_default();
        let genre = CrdtRules::merge(matched, genre);
        let genre = genre.save(library);
        library.save(&GenreRef {
            genre_key: genre.key.clone().unwrap(),
            model_key: release.key.clone().unwrap(),
            ..Default::default()
        });
    }

    release
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
