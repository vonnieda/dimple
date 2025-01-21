use crate::{import::tagged_media_file::TaggedMediaFile, library::Library, merge::CrdtRules, model::{Artist, ArtistRef, Genre, GenreRef, LibraryModel, Model, ModelBasics as _, Release, Track}, plugins::plugin_host::PluginHost};

pub fn refresh_metadata(library: &Library, plugins: &PluginHost, model: &impl LibraryModel) {
    log::info!("refresh_metadata {:?} {:?}", model.type_name(), model.key());
    match model.type_name().as_str() {
        "Track" => {
            if let Some(track) = Track::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &track.clone()) {
                    library.save(&CrdtRules::merge(track, metadata));
                }
            }
        }
        "Artist" => {
            if let Some(artist) = Artist::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &artist.clone()) {
                    library.save(&CrdtRules::merge(artist, metadata));
                }
            }
        }
        "Release" => {
            if let Some(release) = Release::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &release.clone()) {
                    library.save(&CrdtRules::merge(release, metadata));
                }
            }
        }
        "Genre" => {
            if let Some(genre) = Genre::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &genre.clone()) {
                    library.save(&CrdtRules::merge(genre, metadata));
                }
            }
        }
        _ => todo!()
    }
}

// TODO all these match functions can be improved to provide finer matches,
// but I think it all gets replaced with search once I implement Tantivy.

pub fn match_artist(library: &Library, artist: &Artist) -> Option<Artist> {
    library.find("
        SELECT Artist.* 
        FROM Artist 
        WHERE (musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1)
        OR (Artist.name IS NOT NULL AND Artist.name = ?2)
        ", (&artist.musicbrainz_id, &artist.name))
}

pub fn match_genre(library: &Library, genre: &Genre) -> Option<Genre> {
    library.find("
        SELECT Genre.* 
        FROM Genre 
        WHERE (musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1)
        OR (Genre.name IS NOT NULL AND Genre.name = ?2)
        ", (&genre.musicbrainz_id, &genre.name))
}

pub fn match_track(library: &Library, tags: &TaggedMediaFile) -> Option<Track> {
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
pub fn match_release(library: &Library, tags: &TaggedMediaFile) -> Option<Release> {
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

pub fn merge_track(library: &Library, track: &Track, tags: &TaggedMediaFile) -> Track {
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

pub fn merge_release(library: &Library, release: &Release, tags: &TaggedMediaFile) -> Release {
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

