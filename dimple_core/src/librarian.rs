use crate::{import::tagged_media_file::TaggedMediaFile, library::Library, merge::CrdtRules, model::{Artist, ArtistRef, Genre, GenreRef, LibraryModel, Link, Model, ModelBasics as _, Release, Track}, plugins::plugin_host::PluginHost};

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
                    let artist = library.save(&CrdtRules::merge(artist, metadata));
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

pub fn merge_artist(library: &Library, artist: &ArtistMetadata) -> Artist {
    let matched = match_artist(library, &artist.artist).unwrap_or_default();
    let merged = CrdtRules::merge(matched, artist.artist.clone());
    let merged = merged.save(library);
    merge_genres(library, &artist.genres, &merged);
    merged
}

pub fn merge_release(library: &Library, release: &ReleaseMetadata) -> Release {
    let matched = match_release(library, &release).unwrap_or_default();
    let merged = CrdtRules::merge(matched, release.release.clone());
    let merged = merged.save(library);
    merge_artists(library, &release.artists, &merged);
    merge_genres(library, &release.genres, &merged);
    merged
}

pub fn merge_track(library: &Library, track: &TrackMetadata) -> Track {
    let matched = match_track(library, &track).unwrap_or_default();
    let merged = CrdtRules::merge(matched, track.track.clone());
    let mut merged = merged.save(library);
    merge_artists(library, &track.artists, &merged);
    merge_genres(library, &track.genres, &merged);
    if let Some(release) = track.release.clone() {
        let release = merge_release(library, &release);
        merged.release_key = release.key;
        merged = merged.save(&library);
    }
    merged
}

pub fn merge_genre(library: &Library, genre: &Genre) -> Genre {
    let matched = match_genre(library, &genre).unwrap_or_default();
    let genre = CrdtRules::merge(matched, genre.clone());
    genre.save(library)
}

pub fn merge_genres<T: LibraryModel>(library: &Library, genres: &[Genre], model: &T) {
    for genre in genres {
        let genre = merge_genre(library, genre);
        merge_genre_ref(library, &genre, model);
    }
}

pub fn merge_artists<T: LibraryModel>(library: &Library, artists: &[ArtistMetadata], model: &T) {
    for artist in artists {
        let artist = merge_artist(library, &artist);
        merge_artist_ref(library, &artist, model);
    }
}

pub fn merge_artist_ref<T: LibraryModel>(library: &Library, artist: &Artist, model: &T) {
    library.save(&ArtistRef {
        artist_key: artist.key.clone().unwrap(),
        model_key: model.key().clone().unwrap(),
        ..Default::default()
    });
}

pub fn merge_genre_ref<T: LibraryModel>(library: &Library, genre: &Genre, model: &T) {
    library.save(&GenreRef {
        genre_key: genre.key.clone().unwrap(),
        model_key: model.key().clone().unwrap(),
        ..Default::default()
    });
}


// TODO all the match functions can be improved to provide finer matches,
// but I think it all gets replaced with search once I implement Tantivy.

// TODO also, I think these immediately should be modified to check for
// key just in case. 

pub fn match_artist(library: &Library, artist: &Artist) -> Option<Artist> {
    library.find("
        SELECT Artist.* 
        FROM Artist 
        WHERE (musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1)
        OR (Artist.name IS NOT NULL AND Artist.name = ?2 AND Artist.disambiguation = ?3)
        ", (&artist.musicbrainz_id, &artist.name, &artist.disambiguation))
}


// TODO matching hash of embedded artwork might be another good datapoint
pub fn match_release(library: &Library, release: &ReleaseMetadata) -> Option<Release> {
    let matched_release = library.find("
        SELECT Release.* 
        FROM Release 
        WHERE musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1", 
        (&release.release.musicbrainz_id,));
    if matched_release.is_some() {
        return matched_release
    }
    for artist in release.artists.clone() {
        let matched_release: Option<Release> = library.find("
            SELECT r.* FROM Release r
            LEFT JOIN ArtistRef rar ON (rar.model_key = r.key)
            LEFT JOIN Artist ra ON (ra.key = rar.artist_key)
            WHERE (r.title = ?1 AND ra.name = ?2)
            ", (&release.release.title, artist.artist.name));
        if matched_release.is_some() {
            return matched_release
        }
    }
    None
}

pub fn match_track(library: &Library, track: &TrackMetadata) -> Option<Track> {
    // Try to find the track by a unique identifier
    let matched_track = library.find("
        SELECT Track.* 
        FROM Track 
        WHERE musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1", 
        (&track.track.musicbrainz_id,));
    if matched_track.is_some() {
        return matched_track
    }
    // If the track has a Release, search by (artist, album, title)
    if let Some(release) = track.release.clone() {
        for artist in release.artists.clone() {
            let matched_track: Option<Track> = library.find("
                SELECT t.* FROM Track t
                LEFT JOIN Release r ON (r.key = t.release_key)
                LEFT JOIN ArtistRef tar ON (tar.model_key = t.key)
                LEFT JOIN Artist ta ON (ta.key = tar.artist_key)
                LEFT JOIN ArtistRef rar ON (rar.model_key = r.key)
                LEFT JOIN Artist ra ON (ra.key = rar.artist_key)
                WHERE (t.title = ?1 AND r.title = ?2 AND (ta.name = ?3 OR ra.name = ?3))
                ", (&track.track.title, &release.release.title, artist.artist.name));
            if matched_track.is_some() {
                return matched_track
            }
        }
    }
    None
}

pub fn match_genre(library: &Library, genre: &Genre) -> Option<Genre> {
    library.find("
        SELECT Genre.* 
        FROM Genre 
        WHERE (musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1)
        OR (Genre.name IS NOT NULL AND Genre.name = ?2 AND Genre.disambiguation = ?3)
        ", (&genre.musicbrainz_id, &genre.name, &genre.disambiguation))
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ArtistMetadata {
    pub artist: Artist,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ReleaseMetadata {
    pub release: Release,
    pub artists: Vec<ArtistMetadata>,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub tracks: Vec<TrackMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct TrackMetadata {
    pub track: Track,
    pub artists: Vec<ArtistMetadata>,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub release: Option<ReleaseMetadata>,
}
