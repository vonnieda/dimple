use dimple_db::db::transaction::DbTransaction;
use crate::{librarian, library::Library, merge::CrdtRules, model::{Artist, ArtistRef, Dimage, DimageRef, DimpleEntity, Genre, GenreRef, Link, LinkRef, Release, Track}, plugins::plugins::Plugins};

#[derive(Clone)]
pub struct Librarian {
    pub library: Library,
    pub plugins: Plugins,
}

impl Librarian {
    pub fn new(library: &Library, plugins: &Plugins) -> Self {
        Self {
            library: library.clone(),
            plugins: plugins.clone(),
        }
    }
}

pub fn refresh_metadata(library: &Library, plugins: &Plugins, model: &DimpleEntity) {
    match model {
        DimpleEntity::Artist(artist) => {
            for metadata in plugins.artist_metadata(library, &artist) {
                let _ = library.db.transaction(|t| 
                    librarian::merge_artist_metadata(t, &metadata, Some(artist.clone())));
            }
        },
        DimpleEntity::Track(track) => {
            for metadata in plugins.track_metadata(library, &track) {
                let _ = library.db.transaction(|t| 
                    librarian::merge_track_metadata(t, &metadata, Some(track.clone())));
            }
        },
        // DimpleEntity::Genre(genre) => {
        //     if let Some(metadata) = plugins.metadata(library, &genre.clone()) {
        //         library.save(&CrdtRules::merge(genre, metadata));
        //     }
        // },
        DimpleEntity::Release(release) => {
            for metadata in plugins.release_metadata(library, &release) {
                let _ = library.db.transaction(|t| 
                    librarian::merge_release_metadata(t, &metadata, Some(release.clone())));
            }
        },
        _ => todo!()
    }
}

pub fn merge_artist_metadata(txn: &DbTransaction, artist: &ArtistMetadata, pre_match: Option<Artist>) -> Result<Artist, anyhow::Error> {
    let matched = pre_match.or_else(|| match_artist(txn, &artist.artist).ok().flatten()).unwrap_or_default();
    let mut merged = CrdtRules::merge(matched.clone(), artist.artist.clone());
    if merged != matched {
        merged = txn.save(&merged)?;
    }
    merge_genres(txn, &artist.genres, &merged.id)?;
    merge_links(txn, &artist.links, &merged.id)?;
    merge_images(txn, &artist.images, &merged.id)?;
    merge_artist_releases(txn, &artist.releases, &merged)?;
    Ok(merged)
}

pub fn merge_release_metadata(txn: &DbTransaction, metadata: &ReleaseMetadata, pre_match: Option<Release>) -> Result<Release, anyhow::Error> {
    let matched = pre_match.or_else(|| match_release(txn, &metadata).ok().flatten()).unwrap_or_default();
    let mut merged = CrdtRules::merge(matched.clone(), metadata.release.clone());
    if merged != matched {
        merged = txn.save(&merged)?;
    }
    merge_artists(txn, &metadata.artists, &merged.id)?;
    merge_genres(txn, &metadata.genres, &merged.id)?;
    merge_links(txn, &metadata.links, &merged.id)?;
    merge_images(txn, &metadata.images, &merged.id)?;
    merge_release_tracks(txn, &metadata.tracks, &merged)?;
    Ok(merged)
}

pub fn merge_track_metadata(txn: &DbTransaction, metadata: &TrackMetadata, pre_match: Option<Track>) -> Result<Track, anyhow::Error> {
    let matched = pre_match.or_else(|| match_track(txn, &metadata).ok().flatten()).unwrap_or_default();
    let mut merged = CrdtRules::merge(matched.clone(), metadata.track.clone());
    if merged != matched {
        merged = txn.save(&merged)?;
    }
    merge_artists(txn, &metadata.artists, &merged.id)?;
    merge_genres(txn, &metadata.genres, &merged.id)?;
    merge_links(txn, &metadata.links, &merged.id)?;
    merge_images(txn, &metadata.images, &merged.id)?;
    if let Some(release) = metadata.release.clone() {
        let release_match = merged.id.as_ref().and_then(|id| txn.get(id).ok().flatten());
        let release = merge_release_metadata(txn, &release, release_match)?;
        merged.release_id = release.id;
        merged = txn.save(&merged)?;
    }
    Ok(merged)
}

pub fn merge_artist_releases(txn: &DbTransaction, releases: &[ReleaseMetadata], artist: &Artist) -> Result<(), anyhow::Error> {
    for release in releases {
        let release = merge_release_metadata(txn, &release, None)?;
        ArtistRef::attach(txn, artist, &release.id)?;
    }
    Ok(())
}

pub fn merge_release_tracks(txn: &DbTransaction, tracks: &[TrackMetadata], release: &Release) -> Result<(), anyhow::Error> {
    for track in tracks {
        let mut track = merge_track_metadata(txn, &track, None)?;
        if track.release_id.is_none() {
            track.release_id = release.id.clone();
            track = txn.save(&track)?;
        }
    }
    Ok(())
}

pub fn merge_artist(txn: &DbTransaction, artist: &Artist) -> Result<Artist, anyhow::Error> {
    let matched = match_artist(txn, artist)?.unwrap_or_default();
    let merged = CrdtRules::merge(matched.clone(), artist.clone());
    if matched == merged {
        return Ok(matched)
    }
    txn.save(&merged)
}

pub fn merge_link(txn: &DbTransaction, link: &Link) -> Result<Link, anyhow::Error> {
    let matched = match_link(txn, link)?.unwrap_or_default();
    let merged = CrdtRules::merge(matched.clone(), link.clone());
    if matched == merged {
        return Ok(matched)
    }
    txn.save(&merged)
}

pub fn merge_image(txn: &DbTransaction, dimage: &Dimage) -> Result<Dimage, anyhow::Error> {
    let matched = match_dimage(txn, dimage)?.unwrap_or_default();
    let merged = CrdtRules::merge(matched.clone(), dimage.clone());
    if matched == merged {
        return Ok(matched)
    }
    txn.save(&merged)
}

pub fn merge_genre(txn: &DbTransaction, genre: &Genre) -> Result<Genre, anyhow::Error> {
    let matched = match_genre(txn, genre)?.unwrap_or_default();
    let merged = CrdtRules::merge(matched.clone(), genre.clone());
    if matched == merged {
        return Ok(matched)
    }
    txn.save(&merged)
}

pub fn merge_images(txn: &DbTransaction, images: &[Dimage], model_id: &Option<String>) -> Result<(), anyhow::Error> {
    for dimage in images {
        let dimage = merge_image(txn, dimage)?;
        DimageRef::attach(txn, &dimage, model_id)?;
    }
    Ok(())
}

pub fn merge_links(txn: &DbTransaction, links: &[Link], model_id: &Option<String>) -> Result<(), anyhow::Error> {
    for link in links {
        let link = merge_link(txn, link)?;
        LinkRef::attach(txn, &link, model_id)?;
    }
    Ok(())
}

pub fn merge_genres(txn: &DbTransaction, genres: &[Genre], model_id: &Option<String>) -> Result<(), anyhow::Error> {
    for genre in genres {
        let genre = merge_genre(txn, genre)?;
        GenreRef::attach(txn, &genre, model_id)?;
    }
    Ok(())
}

pub fn merge_artists(txn: &DbTransaction, artists: &[ArtistMetadata], model_id: &Option<String>) -> Result<(), anyhow::Error> {
    for artist in artists {
        let artist = merge_artist_metadata(txn, &artist, None)?;
        ArtistRef::attach(txn, &artist, model_id)?;
    }
    Ok(())
}

pub fn match_artist(txn: &DbTransaction, artist: &Artist) -> Result<Option<Artist>, anyhow::Error> {
    let results = txn.query("
        SELECT Artist.* 
        FROM Artist 
        WHERE (Artist.musicbrainz_id IS NOT NULL AND Artist.musicbrainz_id = ?1)
        OR (Artist.name IS NOT NULL AND Artist.name COLLATE NOCASE = ?2 AND ((Artist.disambiguation IS NULL AND ?3 IS NULL) OR (Artist.disambiguation = ?3)))
        ", (&artist.musicbrainz_id, &artist.name, &artist.disambiguation))?;
    Ok(results.into_iter().next())
}

pub fn match_genre(txn: &DbTransaction, genre: &Genre) -> Result<Option<Genre>, anyhow::Error> {
    let results = txn.query("
        SELECT Genre.* 
        FROM Genre 
        WHERE (Genre.musicbrainz_id IS NOT NULL AND Genre.musicbrainz_id = ?1)
        OR (Genre.name IS NOT NULL AND Genre.name COLLATE NOCASE = ?2 AND ((Genre.disambiguation IS NULL AND ?3 IS NULL) OR (Genre.disambiguation COLLATE NOCASE = ?3)))
        ", (&genre.musicbrainz_id, &genre.name, &genre.disambiguation))?;
    Ok(results.into_iter().next())
}

pub fn match_link(txn: &DbTransaction, link: &Link) -> Result<Option<Link>, anyhow::Error> {
    let results = txn.query("
        SELECT Link.* 
        FROM Link 
        WHERE (Link.url = ?1)
        ", (&link.url,))?;
    Ok(results.into_iter().next())
}

pub fn match_dimage(txn: &DbTransaction, dimage: &Dimage) -> Result<Option<Dimage>, anyhow::Error> {
    let results = txn.query("
        SELECT Dimage.* 
        FROM Dimage 
        WHERE (Dimage.sha256 = ?1)
        ", (&dimage.sha256,))?;
    Ok(results.into_iter().next())
}

pub fn match_release(txn: &DbTransaction, release: &ReleaseMetadata) -> Result<Option<Release>, anyhow::Error> {
    let matched_release: Vec<Release> = txn.query("
        SELECT Release.* 
        FROM Release 
        WHERE Release.musicbrainz_id IS NOT NULL AND Release.musicbrainz_id = ?1", 
        (&release.release.musicbrainz_id,))?;
    if !matched_release.is_empty() {
        return Ok(matched_release.into_iter().next())
    }
    for artist in release.artists.clone() {
        let matched_release: Vec<Release> = txn.query("
            SELECT r.* FROM Release r
            LEFT JOIN ArtistRef rar ON (rar.model_id = r.id)
            LEFT JOIN Artist ra ON (ra.id = rar.artist_id)
            WHERE (
                r.title COLLATE NOCASE = ?1 
                AND ra.name COLLATE NOCASE = ?2
            )
            ", (&release.release.title, artist.artist.name))?;
        if !matched_release.is_empty() {
            return Ok(matched_release.into_iter().next())
        }
    }
    Ok(None)
}


pub fn match_track(txn: &DbTransaction, track: &TrackMetadata) -> Result<Option<Track>, anyhow::Error> {
    // Try to find the track by a unique identifier
    let matched_track: Vec<Track> = txn.query("
        SELECT Track.* 
        FROM Track 
        WHERE musicbrainz_id IS NOT NULL AND musicbrainz_id = ?1", 
        (&track.track.musicbrainz_id,))?;
    if !matched_track.is_empty() {
        return Ok(matched_track.into_iter().next())
    }
    // If the track has a Release, search by (artist, album, title)
    if let Some(release) = track.release.clone() {
        for artist in release.artists.clone() {
            let matched_track: Vec<Track> = txn.query("
                SELECT t.* FROM Track t
                LEFT JOIN Release r ON (r.id = t.release_id)
                LEFT JOIN ArtistRef tar ON (tar.model_id = t.id)
                LEFT JOIN Artist ta ON (ta.id = tar.artist_id)
                LEFT JOIN ArtistRef rar ON (rar.model_id = r.id)
                LEFT JOIN Artist ra ON (ra.id = rar.artist_id)
                WHERE (
                    t.title COLLATE NOCASE = ?1 
                    AND COLLATE NOCASE r.title = ?2 
                    AND (ta.name COLLATE NOCASE = ?3 OR ra.name COLLATE NOCASE = ?3)
                )
                ", (&track.track.title, &release.release.title, artist.artist.name))?;
            if !matched_track.is_empty() {
                return Ok(matched_track.into_iter().next())
            }
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ArtistMetadata {
    pub artist: Artist,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub images: Vec<Dimage>,
    pub releases: Vec<ReleaseMetadata>,
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
    // TODO Change each to *Metadata so we have more to merge.
    pub tracks: Vec<Track>,
    pub artists: Vec<Artist>,
    pub releases: Vec<Release>,
    pub genres: Vec<Genre>,
}

mod tests {
    use crate::{librarian::{self, ArtistMetadata}, library::Library, model::Artist};

    #[test]
    fn test_merge_artist_metadata() {
        let library = Library::open_memory();
        let (artist1, artist2, artist3, artist4) = library.db.transaction(|txn| {
            let artist1 = librarian::merge_artist_metadata(txn, &ArtistMetadata {
                artist: Artist {
                    name: Some("Something Cool".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }, None)?;
            dbg!(&artist1);
            let artist2 = librarian::merge_artist_metadata(txn, &ArtistMetadata {
                artist: Artist {
                    name: Some("Something Cool".to_string()),
                    musicbrainz_id: Some("4563463".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }, None)?;
            dbg!(&artist2);
            let artist3 = librarian::merge_artist_metadata(txn, &ArtistMetadata {
                artist: Artist {
                    name: Some("Something Cool".to_string()),
                    disambiguation: Some("the other one".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }, None)?;
            dbg!(&artist3);
            let artist4 = librarian::merge_artist_metadata(txn, &ArtistMetadata {
                artist: Artist {
                    name: Some("Something Cool".to_string()),
                    disambiguation: Some("the other one".to_string()),
                    musicbrainz_id: Some("123123".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }, None)?;
            Ok((artist1, artist2, artist3, artist4))
        }).unwrap();
        assert!(artist1.id != artist3.id);
        assert!(artist1.id == artist2.id);
        assert!(artist3.id == artist4.id);
    }
}