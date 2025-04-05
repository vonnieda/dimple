use crate::{librarian, library::Library, merge::CrdtRules, model::{Artist, ArtistRef, Dimage, DimageRef, Genre, GenreRef, LibraryModel, Link, LinkRef, Model, ModelBasics as _, Release, Track}, plugins::plugin_host::PluginHost};

pub fn refresh_metadata(library: &Library, plugins: &PluginHost, model: &impl LibraryModel) {
    log::info!("refresh_metadata {:?} {:?}", model.type_name(), model.key());
    match model.type_name().as_str() {
        "Track" => {
            if let Some(track) = Track::get(library, &model.key().clone().unwrap()) {
                for metadata in plugins.track_metadata(library, &track) {
                    librarian::merge_track_metadata(library, &metadata, Some(track.clone()));
                }
            }
        }
        "Artist" => {
            if let Some(artist) = Artist::get(library, &model.key().clone().unwrap()) {
                for metadata in plugins.artist_metadata(library, &artist) {
                    librarian::merge_artist_metadata(library, &metadata, Some(artist.clone()));
                }
            }
        }
        "Release" => {
            if let Some(release) = Release::get(library, &model.key().clone().unwrap()) {
                for metadata in plugins.release_metadata(library, &release) {
                    librarian::merge_release_metadata(library, &metadata, Some(release.clone()));
                }
            }
        }
        // "Genre" => {
        //     if let Some(genre) = Genre::get(library, &model.key().clone().unwrap()) {
        //         if let Some(metadata) = plugins.metadata(library, &genre.clone()) {
        //             library.save(&CrdtRules::merge(genre, metadata));
        //         }
        //     }
        // }
        _ => todo!()
    }
}

/// Okay, so the way I /want/ search to work is the library search happens realtime
/// and then no more than once per second, we do the plugin search. All the results
/// are merged together and then we continuously see the updated results.
pub fn search(library: &Library, plugins: &PluginHost, query: &str) -> SearchResults {
    // here we go again, but joyfully
    let plugin_results = plugins.search(library, query);

    for result in plugin_results {
        for artist in result.artists {
            librarian::merge_artist(library, &artist);
        }
    }

    let query = format!("%{}%", query);
    let artists = Artist::query(library, 
        "SELECT * FROM Artist WHERE name LIKE ?1 LIMIT 25", (&query,));
    let releases = Release::query(library, 
        "SELECT * FROM Release WHERE title LIKE ?1 LIMIT 25", (&query,));
    let genres = Genre::query(library, 
        "SELECT * FROM Genre WHERE name LIKE ?1 LIMIT 25", (&query,));
    let tracks = Track::query(library, 
        "SELECT * FROM Track WHERE title LIKE ?1 LIMIT 25", (&query,));    

    SearchResults { 
        artists, 
        releases, 
        genres, 
        tracks, 
        ..Default::default()
    }
}

pub fn merge_artist(library: &Library, artist: &Artist) -> Artist {
    let matched = match_artist(library, artist).unwrap_or_default();
    let merged = CrdtRules::merge(matched, artist.clone());
    merged.save(library)
}

pub fn merge_artist_metadata(library: &Library, artist: &ArtistMetadata, pre_match: Option<Artist>) -> Artist {
    let matched = pre_match.or_else(|| match_artist(library, &artist.artist)).unwrap_or_default();
    let merged = CrdtRules::merge(matched, artist.artist.clone());
    let merged = merged.save(library);
    merge_genres(library, &artist.genres, &merged);
    merge_links(library, &artist.links, &merged);
    merge_images(library, &artist.images, &merged);
    merged
}

// two things: we shouldn't be creating a release if there's no release info
// at all
// and even if we do, we need pull it first like we did the track - okay fixed that
// do others need that treatment?
pub fn merge_release_metadata(library: &Library, metadata: &ReleaseMetadata, pre_match: Option<Release>) -> Release {
    let matched = pre_match.or_else(|| match_release(library, &metadata)).unwrap_or_default();
    let merged = CrdtRules::merge(matched, metadata.release.clone());
    let merged = merged.save(library);
    merge_artists(library, &metadata.artists, &merged);
    merge_genres(library, &metadata.genres, &merged);
    merge_links(library, &metadata.links, &merged);
    merge_images(library, &metadata.images, &merged);
    merged
}

pub fn merge_track_metadata(library: &Library, metadata: &TrackMetadata, pre_match: Option<Track>) -> Track {
    let matched = pre_match.or_else(|| match_track(library, &metadata)).unwrap_or_default();
    let merged = CrdtRules::merge(matched, metadata.track.clone());
    let mut merged = merged.save(library);
    merge_artists(library, &metadata.artists, &merged);
    merge_genres(library, &metadata.genres, &merged);
    merge_links(library, &metadata.links, &merged);
    merge_images(library, &metadata.images, &merged);
    if let Some(release) = metadata.release.clone() {
        let release = merge_release_metadata(library, &release, merged.release(library));
        merged.release_key = release.key;
        merged = merged.save(&library);
    }
    merged
}

pub fn merge_link(library: &Library, link: &Link) -> Link {
    let matched = match_link(library, &link).unwrap_or_default();
    let link = CrdtRules::merge(matched, link.clone());
    link.save(library)
}

pub fn merge_images<T: LibraryModel>(library: &Library, images: &[Dimage], model: &T) {
    for dimage in images {
        let dimage = merge_image(library, dimage);
        DimageRef::attach(library, &dimage, model);
    }
}

pub fn merge_image(library: &Library, dimage: &Dimage) -> Dimage {
    let matched = match_dimage(library, &dimage).unwrap_or_default();
    let dimage = CrdtRules::merge(matched, dimage.clone());
    dimage.save(library)
}

pub fn merge_links<T: LibraryModel>(library: &Library, links: &[Link], model: &T) {
    for link in links {
        let link = merge_link(library, link);
        merge_link_ref(library, &link, model);
    }
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
        let artist = merge_artist_metadata(library, &artist, None);
        merge_artist_ref(library, &artist, model);
    }
}

pub fn merge_artist_ref<T: LibraryModel>(library: &Library, artist: &Artist, model: &T) {
    ArtistRef::attach(library, artist, model);
}

pub fn merge_genre_ref<T: LibraryModel>(library: &Library, genre: &Genre, model: &T) {
    GenreRef::attach(library, genre, model);
}

pub fn merge_link_ref<T: LibraryModel>(library: &Library, link: &Link, model: &T) {
    LinkRef::attach(library, link, model);
}

pub fn match_artist(library: &Library, artist: &Artist) -> Option<Artist> {
    library.find("
        SELECT Artist.* 
        FROM Artist 
        WHERE (Artist.musicbrainz_id IS NOT NULL AND Artist.musicbrainz_id = ?1)
        OR (Artist.name IS NOT NULL AND Artist.name = ?2 AND ((Artist.disambiguation IS NULL AND ?3 IS NULL) OR (Artist.disambiguation = ?3)))
        ", (&artist.musicbrainz_id, &artist.name, &artist.disambiguation))
}

pub fn match_release(library: &Library, release: &ReleaseMetadata) -> Option<Release> {
    let matched_release = library.find("
        SELECT Release.* 
        FROM Release 
        WHERE Release.musicbrainz_id IS NOT NULL AND Release.musicbrainz_id = ?1", 
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
        WHERE (Genre.musicbrainz_id IS NOT NULL AND Genre.musicbrainz_id = ?1)
        OR (Genre.name IS NOT NULL AND Genre.name = ?2 COLLATE NOCASE AND ((Genre.disambiguation IS NULL AND ?3 IS NULL) OR (Genre.disambiguation = ?3 COLLATE NOCASE)))
        ", (&genre.musicbrainz_id, &genre.name, &genre.disambiguation))
}

pub fn match_link(library: &Library, link: &Link) -> Option<Link> {
    library.find("
        SELECT Link.* 
        FROM Link 
        WHERE (Link.url = ?1)
        ", (&link.url,))
}

pub fn match_dimage(library: &Library, dimage: &Dimage) -> Option<Dimage> {
    library.find("
        SELECT Dimage.* 
        FROM Dimage 
        WHERE (Dimage.sha256 = ?1)
        ", (&dimage.sha256,))
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ArtistMetadata {
    pub artist: Artist,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub images: Vec<Dimage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ReleaseMetadata {
    pub release: Release,
    pub artists: Vec<ArtistMetadata>,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub tracks: Vec<TrackMetadata>,
    pub images: Vec<Dimage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct TrackMetadata {
    pub track: Track,
    pub artists: Vec<ArtistMetadata>,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub release: Option<ReleaseMetadata>,
    pub images: Vec<Dimage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
    pub artists: Vec<Artist>,
    pub releases: Vec<Release>,
    pub genres: Vec<Genre>,
}

mod tests {
    use crate::{librarian::{self, ArtistMetadata}, library::Library, model::Artist};

    #[test]
    fn merge_artist_metadata() {
        let library = Library::open_memory();
        let artist1 = librarian::merge_artist_metadata(&library, &ArtistMetadata {
            artist: Artist {
                name: Some("Something Cool".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }, None);
        dbg!(&artist1);
        let artist2 = librarian::merge_artist_metadata(&library, &ArtistMetadata {
            artist: Artist {
                name: Some("Something Cool".to_string()),
                musicbrainz_id: Some("4563463".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }, None);
        dbg!(&artist2);
        let artist3 = librarian::merge_artist_metadata(&library, &ArtistMetadata {
            artist: Artist {
                name: Some("Something Cool".to_string()),
                disambiguation: Some("the other one".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }, None);
        dbg!(&artist3);
        let artist4 = librarian::merge_artist_metadata(&library, &ArtistMetadata {
            artist: Artist {
                name: Some("Something Cool".to_string()),
                disambiguation: Some("the other one".to_string()),
                musicbrainz_id: Some("123123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }, None);
        assert!(artist1.key != artist3.key);
        assert!(artist1.key == artist2.key);
        assert!(artist3.key == artist4.key);
    }
}