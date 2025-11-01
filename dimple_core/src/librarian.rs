use dimple_db::db::transaction::DbTransaction;
use crate::{library::Library, merge_rules::MergeExtend, model::{Artist, ArtistRef, Dimage, DimageRef, DimpleEntity, Genre, GenreRef, Link, LinkRef, Recording, Release, ReleaseGroup, ReleaseGroupSecondaryType, ReleaseGroupSecondaryTypeRef, Track}, plugins::plugins::Plugins};

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
    // TODO log these errors
    match model {
        DimpleEntity::Artist(artist) => {
            for metadata in plugins.artist_metadata(library, artist) {
                library.db.transaction(|t| {
                    merge_artist_metadata(t, &artist, &metadata)
                }).unwrap();
            }
            for metadata in plugins.artist_release_groups(library, artist) {
                library.db.transaction(|t| {
                    let release_group = merge_artist_release_group(t, &artist, &metadata.release_group)?;       
                    merge_release_group_metadata(t, &release_group, &metadata)
                }).unwrap();
            }
        },
        DimpleEntity::ReleaseGroup(release_group) => {
            for metadata in plugins.release_group_metadata(library, release_group) {
                library.db.transaction(|t| {
                    merge_release_group_metadata(t, &release_group, &metadata)
                }).unwrap();
            }
            for metadata in plugins.release_group_releases(library, release_group) {
                library.db.transaction(|t| {
                    let release = merge_release_group_release(t, &release_group, &metadata.release)?;
                    merge_release_metadata(t, &release, &metadata)
                }).unwrap();
            }
        },
        DimpleEntity::Release(release) => {
            for metadata in plugins.release_metadata(library, release) {
                library.db.transaction(|t| {
                    merge_release_metadata(t, release, &metadata)
                }).unwrap();
            }
        },
        DimpleEntity::Track(track) => {
            for metadata in plugins.track_metadata(library, track) {
                library.db.transaction(|t| {
                    merge_track_metadata(t, track, &metadata)
                }).unwrap();
            }
        },
        _ => todo!()
    }
}

/// Find or create an Artist that matches all of the identifying fields of
/// the given Artist
pub fn merge_artist(txn: &DbTransaction, artist: &Artist) -> Result<Artist, anyhow::Error> {
    let sql = "
        SELECT Artist.*
        FROM Artist
        WHERE (Artist.name IS NULL OR Artist.name COLLATE NOCASE = ?1)
        AND (?2 IS NULL OR Artist.disambiguation IS NULL OR Artist.disambiguation COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR Artist.country IS NULL OR Artist.country COLLATE NOCASE = ?3)
        AND (?4 IS NULL OR Artist.discogs_id IS NULL OR Artist.discogs_id = ?4)
        AND (?5 IS NULL OR Artist.lastfm_id IS NULL OR Artist.lastfm_id = ?5)
        AND (?6 IS NULL OR Artist.musicbrainz_id IS NULL OR Artist.musicbrainz_id = ?6)
        AND (?7 IS NULL OR Artist.spotify_id IS NULL OR Artist.spotify_id = ?7)
    ";
    let matched = txn.find::<Artist, _>(&sql, (&artist.name,
        &artist.disambiguation,
        &artist.country,
        &artist.discogs_id,
        &artist.lastfm_id,
        &artist.musicbrainz_id,
        &artist.spotify_id))?.unwrap_or_default();
    let merged = matched.merge_extend(artist);
    txn.save(&merged)
}

pub fn merge_artist_metadata(txn: &DbTransaction, artist: &Artist, metadata: &ArtistMetadata) -> Result<Artist, anyhow::Error> {
    let mut merged = artist.merge_extend(&metadata.artist);
    if merged != *artist {
        merged = txn.save(&merged)?;
    }
    merge_entity_genres(txn, &merged.id, &metadata.genres)?;
    merge_entity_links(txn, &merged.id, &metadata.links)?;
    merge_entity_images(txn, &merged.id, &metadata.images)?;
    Ok(merged)
}

/// Find or create a ReleaseGroup that matches all of the identifying fields of
/// the given ReleaseGroup, scoped to a specific Artist.
pub fn merge_artist_release_group(txn: &DbTransaction, artist: &Artist,
    release_group: &ReleaseGroup)
    -> Result<ReleaseGroup, anyhow::Error> {

    let sql = "
        SELECT ReleaseGroup.*
        FROM ReleaseGroup
        JOIN ArtistRef ON (ReleaseGroup.id = ArtistRef.model_id)
        WHERE ArtistRef.artist_id = ?1
        AND (ReleaseGroup.title IS NULL OR ReleaseGroup.title COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR ReleaseGroup.disambiguation IS NULL OR ReleaseGroup.disambiguation COLLATE NOCASE = ?3)
        AND (?4 IS NULL OR ReleaseGroup.discogs_id IS NULL OR ReleaseGroup.discogs_id = ?4)
        AND (?5 IS NULL OR ReleaseGroup.lastfm_id IS NULL OR ReleaseGroup.lastfm_id = ?5)
        AND (?6 IS NULL OR ReleaseGroup.musicbrainz_id IS NULL OR ReleaseGroup.musicbrainz_id = ?6)
        AND (?7 IS NULL OR ReleaseGroup.spotify_id IS NULL OR ReleaseGroup.spotify_id = ?7)
    ";

    let matched = txn.find::<ReleaseGroup, _>(&sql, (&artist.id,
        &release_group.title,
        &release_group.disambiguation,
        &release_group.discogs_id,
        &release_group.lastfm_id,
        &release_group.musicbrainz_id,
        &release_group.spotify_id))?.unwrap_or_default();
    let merged = matched.merge_extend(release_group);
    let saved = txn.save(&merged)?;
    ArtistRef::attach(txn, artist, &saved.id)?;
    Ok(saved)
}

/// Find or create a Release that matches all of the identifying fields of
/// the given Release, scoped to a specific Artist.
pub fn merge_artist_release(txn: &DbTransaction, artist: &Artist,
    release: &Release)
    -> Result<Release, anyhow::Error> {

    let sql = "
        SELECT Release.*
        FROM Release
        JOIN ArtistRef ON (Release.id = ArtistRef.model_id)
        WHERE ArtistRef.artist_id = ?1
        AND (Release.title IS NULL OR Release.title COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR Release.disambiguation IS NULL OR Release.disambiguation COLLATE NOCASE = ?3)
        AND (?4 IS NULL OR Release.discogs_id IS NULL OR Release.discogs_id = ?4)
        AND (?5 IS NULL OR Release.lastfm_id IS NULL OR Release.lastfm_id = ?5)
        AND (?6 IS NULL OR Release.musicbrainz_id IS NULL OR Release.musicbrainz_id = ?6)
        AND (?7 IS NULL OR Release.spotify_id IS NULL OR Release.spotify_id = ?7)
    ";

    let matched = txn.find::<Release, _>(&sql, (&artist.id,
        &release.title,
        &release.disambiguation,
        &release.discogs_id,
        &release.lastfm_id,
        &release.musicbrainz_id,
        &release.spotify_id))?.unwrap_or_default();
    let merged = matched.merge_extend(release);
    let saved = txn.save(&merged)?;
    ArtistRef::attach(txn, artist, &saved.id)?;
    Ok(saved)
}

/// Find or create a ReleaseGroup that matches all of the identifying fields of
/// the given ReleaseGroup.
///
/// TODO This is sus, cause we should at least be trying for an artist.
/// This is used by search to merge ReleaseGroupMetadata, and I think we
/// probably want to change that to use artist. Or something. I dunno.
/// Could just use first artist credit, and abort if none
pub fn merge_release_group(txn: &DbTransaction, release_group: &ReleaseGroup) -> anyhow::Result<ReleaseGroup> {
    let sql = "
        SELECT ReleaseGroup.*
        FROM ReleaseGroup
        WHERE (ReleaseGroup.title IS NULL OR ReleaseGroup.title COLLATE NOCASE = ?1)
        AND (?2 IS NULL OR ReleaseGroup.disambiguation IS NULL OR ReleaseGroup.disambiguation COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR ReleaseGroup.discogs_id IS NULL OR ReleaseGroup.discogs_id = ?3)
        AND (?4 IS NULL OR ReleaseGroup.lastfm_id IS NULL OR ReleaseGroup.lastfm_id = ?4)
        AND (?5 IS NULL OR ReleaseGroup.musicbrainz_id IS NULL OR ReleaseGroup.musicbrainz_id = ?5)
        AND (?6 IS NULL OR ReleaseGroup.spotify_id IS NULL OR ReleaseGroup.spotify_id = ?6)
    ";

    let matched = txn.find::<ReleaseGroup, _>(&sql,
        (&release_group.title,
        &release_group.disambiguation,
        &release_group.discogs_id,
        &release_group.lastfm_id,
        &release_group.musicbrainz_id,
        &release_group.spotify_id))?.unwrap_or_default();
    let merged = matched.merge_extend(release_group);
    txn.save(&merged)
}

pub fn merge_release_group_metadata(txn: &DbTransaction, release_group: &ReleaseGroup, metadata: &ReleaseGroupMetadata) -> Result<ReleaseGroup, anyhow::Error> {
    let mut merged = release_group.merge_extend(&metadata.release_group);
    if merged != *release_group {
        merged = txn.save(&merged)?;
    }
    for secondary_type in metadata.secondary_types.iter() {
        let _ = ReleaseGroupSecondaryTypeRef::attach(txn, &merged, secondary_type.clone());
    }
    merge_entity_artists(txn, &merged.id, &metadata.artists)?;
    merge_entity_genres(txn, &merged.id, &metadata.genres)?;
    merge_entity_links(txn, &merged.id, &metadata.links)?;
    merge_entity_images(txn, &merged.id, &metadata.images)?;
    Ok(merged)
}

/// Find or create a Release that matches all of the identifying fields of
/// the given Release, scoped to a specific ReleaseGroup.
pub fn merge_release_group_release(txn: &DbTransaction, release_group: &ReleaseGroup, release: &Release) -> Result<Release, anyhow::Error> {
    let sql = "
        SELECT Release.*
        FROM Release
        WHERE Release.release_group_id = ?1
        AND (Release.title IS NULL OR Release.title COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR Release.disambiguation IS NULL OR Release.disambiguation COLLATE NOCASE = ?3)
        AND (?4 IS NULL OR Release.country IS NULL OR Release.country COLLATE NOCASE = ?4)
        AND (?5 IS NULL OR Release.discogs_id IS NULL OR Release.discogs_id = ?5)
        AND (?6 IS NULL OR Release.lastfm_id IS NULL OR Release.lastfm_id = ?6)
        AND (?7 IS NULL OR Release.musicbrainz_id IS NULL OR Release.musicbrainz_id = ?7)
        AND (?8 IS NULL OR Release.spotify_id IS NULL OR Release.spotify_id = ?8)
    ";

    let matched = txn.find::<Release, _>(&sql, (&release_group.id,
        &release.title,
        &release.disambiguation,
        &release.country,
        &release.discogs_id,
        &release.lastfm_id,
        &release.musicbrainz_id,
        &release.spotify_id))?.unwrap_or_default();
    let mut merged = matched.merge_extend(release);
    merged.release_group_id = release_group.id.clone();
    txn.save(&merged)
}

pub fn merge_release_metadata(txn: &DbTransaction, release: &Release, metadata: &ReleaseMetadata) -> Result<Release, anyhow::Error> {
    let mut merged = release.merge_extend(&metadata.release);
    if merged != *release {
        merged = txn.save(&merged)?;
    }
    merge_entity_artists(txn, &merged.id, &metadata.artists)?;
    merge_entity_genres(txn, &merged.id, &metadata.genres)?;
    merge_entity_links(txn, &merged.id, &metadata.links)?;
    merge_entity_images(txn, &merged.id, &metadata.images)?;
    for track_metadata in metadata.tracks.iter() {
        let track = merge_release_track(txn, release, &track_metadata.track)?;
        let _ = merge_track_metadata(txn, &track, &track_metadata)?;
    }
    Ok(merged)
}

/// Find or create a Track that matches all of the identifying fields of
/// the given Track, scoped to a specific Release.
pub fn merge_release_track(txn: &DbTransaction, release: &Release, track: &Track) -> Result<Track, anyhow::Error> {
    let sql = "
        SELECT Track.*
        FROM Track
        WHERE Track.release_id = ?1
        AND (Track.title IS NULL OR Track.title COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR Track.disambiguation IS NULL OR Track.disambiguation COLLATE NOCASE = ?3)
        AND (?4 IS NULL OR Track.discogs_id IS NULL OR Track.discogs_id = ?4)
        AND (?5 IS NULL OR Track.lastfm_id IS NULL OR Track.lastfm_id = ?5)
        AND (?6 IS NULL OR Track.musicbrainz_id IS NULL OR Track.musicbrainz_id = ?6)
        AND (?7 IS NULL OR Track.spotify_id IS NULL OR Track.spotify_id = ?7)
    ";

    let matched = txn.find::<Track, _>(&sql, (&release.id,
        &track.title,
        &track.disambiguation,
        &track.discogs_id,
        &track.lastfm_id,
        &track.musicbrainz_id,
        &track.spotify_id))?.unwrap_or_default();
    let mut merged = matched.merge_extend(track);
    merged.release_id = release.id.clone();
    txn.save(&merged)
}


pub fn merge_track_metadata(txn: &DbTransaction, track: &Track, metadata: &TrackMetadata) -> Result<Track, anyhow::Error> {
    let mut merged = track.merge_extend(&metadata.track);
    if merged != *track {
        merged = txn.save(&merged)?;
    }
    merge_entity_artists(txn, &merged.id, &metadata.artists)?;
    merge_entity_genres(txn, &merged.id, &metadata.genres)?;
    merge_entity_links(txn, &merged.id, &metadata.links)?;
    merge_entity_images(txn, &merged.id, &metadata.images)?;
    Ok(merged)
}

pub fn merge_genre(txn: &DbTransaction, genre: &Genre) -> Result<Genre, anyhow::Error> {
    let sql = "
        SELECT Genre.*
        FROM Genre
        WHERE (Genre.name IS NULL OR Genre.name COLLATE NOCASE = ?1)
        AND (?2 IS NULL OR Genre.disambiguation IS NULL OR Genre.disambiguation COLLATE NOCASE = ?2)
        AND (?3 IS NULL OR Genre.discogs_id IS NULL OR Genre.discogs_id = ?3)
        AND (?4 IS NULL OR Genre.lastfm_id IS NULL OR Genre.lastfm_id = ?4)
        AND (?5 IS NULL OR Genre.musicbrainz_id IS NULL OR Genre.musicbrainz_id = ?5)
        AND (?6 IS NULL OR Genre.spotify_id IS NULL OR Genre.spotify_id = ?6)
    ";
    let matched = txn.find::<Genre, _>(&sql, (&genre.name,
        &genre.disambiguation,
        &genre.discogs_id,
        &genre.lastfm_id,
        &genre.musicbrainz_id,
        &genre.spotify_id))?.unwrap_or_default();
    let merged = matched.merge_extend(genre);
    txn.save(&merged)
}

fn merge_entity_artists(txn: &DbTransaction, entity_id: &Option<String>, artists: &[ArtistMetadata]) -> Result<(), anyhow::Error> {
    for metadata in artists {
        let artist = merge_artist(txn, &metadata.artist)?;
        let artist = merge_artist_metadata(txn, &artist, &metadata)?;
        ArtistRef::attach(txn, &artist, entity_id)?;
    }
    Ok(())
}

fn merge_entity_images(txn: &DbTransaction, entity_id: &Option<String>, images: &[Dimage]) -> Result<(), anyhow::Error> {
    for dimage in images {
        let dimage = merge_dimage(txn, dimage)?;
        DimageRef::attach(txn, &dimage, entity_id)?;
    }
    Ok(())
}

fn merge_entity_links(txn: &DbTransaction, entity_id: &Option<String>, links: &[Link]) -> Result<(), anyhow::Error> {
    for link in links {
        let link = merge_link(txn, link)?;
        LinkRef::attach(txn, &link, entity_id)?;
    }
    Ok(())
}

fn merge_entity_genres(txn: &DbTransaction, entity_id: &Option<String>, genres: &[Genre]) -> Result<(), anyhow::Error> {
    for genre in genres {
        let genre = merge_genre(txn, genre)?;
        GenreRef::attach(txn, &genre, entity_id)?;
    }
    Ok(())
}

fn merge_link(txn: &DbTransaction, link: &Link) -> Result<Link, anyhow::Error> {
    let sql = "
        SELECT Link.* 
        FROM Link 
        WHERE (Link.url = ?1)
    ";
    let matched = txn.find::<Link, _>(&sql, (&link.url,))?.unwrap_or_default();
    let merged = matched.merge_extend(link);
    txn.save(&merged)
}

fn merge_dimage(txn: &DbTransaction, dimage: &Dimage) -> Result<Dimage, anyhow::Error> {
    let sql = "
        SELECT Dimage.* 
        FROM Dimage 
        WHERE (Dimage.sha256 = ?1)
    ";
    let matched = txn.find::<Dimage, _>(&sql, (&dimage.sha256,))?.unwrap_or_default();
    let merged = matched.merge_extend(dimage);
    txn.save(&merged)
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ArtistMetadata {
    pub artist: Artist,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub images: Vec<Dimage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ReleaseGroupMetadata {
    pub release_group: ReleaseGroup,
    pub secondary_types: Vec<ReleaseGroupSecondaryType>,
    pub artists: Vec<ArtistMetadata>,
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
    pub images: Vec<Dimage>,
    pub tracks: Vec<TrackMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct TrackMetadata {
    pub track: Track,
    pub artists: Vec<ArtistMetadata>,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub images: Vec<Dimage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct RecordingMetadata {
    pub recording: Recording,
    pub artists: Vec<ArtistMetadata>,
    pub genres: Vec<Genre>,
    pub links: Vec<Link>,
    pub images: Vec<Dimage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct SearchResults {
    pub tracks: Vec<TrackMetadata>,
    pub artists: Vec<ArtistMetadata>,
    pub releases: Vec<ReleaseMetadata>,
    pub release_groups: Vec<ReleaseGroupMetadata>,
    pub genres: Vec<Genre>,
}

mod tests {
    use crate::{librarian::{self, merge_artist, TrackMetadata}, library::Library, model::{Artist, Genre, ModelBasics, Release, ReleaseGroup, Track}};

    #[test]
    fn smoke_test() -> anyhow::Result<()>{
        let library = Library::open_memory();
        let artist = library.db.transaction(|txn| {
            librarian::merge_artist(txn, &Artist {
                name: Some("Wyld Stallions".to_string()),
                ..Default::default()
            })
        })?;
        let release_group = library.db.transaction(|txn| {
            librarian::merge_artist_release_group(txn, &artist, &ReleaseGroup {
                title: Some("Epic Anthems".to_string()),
                ..Default::default()
            })
        })?;
        let release = library.db.transaction(|txn| {
            librarian::merge_release_group_release(txn, &release_group, &Release {
                title: Some("Epic Anthems".to_string()),
                country: Some("us".to_string()),
                ..Default::default()
            })
        })?;
        let (track1, track2, track3) = library.db.transaction(|txn| {
            let track1 = librarian::merge_release_track(txn, &release, &Track {
                title: Some("Station".to_string()),
                position: Some(1),
                ..Default::default()
            })?;
            let track2 = librarian::merge_release_track(txn, &release, &Track {
                title: Some("Straight to Heck".to_string()),
                position: Some(2),
                ..Default::default()
            })?;
            let track3 = librarian::merge_release_track(txn, &release, &Track {
                title: Some("Don't Fear Mr. Grim Reaper Man".to_string()),
                position: Some(3),
                ..Default::default()
            })?;
            Ok((track1, track2, track3))
        })?;

        assert_eq!(release_group.artist(&library).unwrap().id, artist.id);
        assert_eq!(release.release_group_id, release_group.id);
        assert_eq!(track1.release_id, release.id);
        assert_eq!(track2.release_id, release.id);
        assert_eq!(track3.release_id, release.id);
        Ok(())
    }

    #[test]
    fn test_merge_artist() -> anyhow::Result<()> {
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            librarian::merge_artist(txn, &Artist {
                name: Some("Wyld Stallions".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist(txn, &Artist {
                name: Some("Wyld Stallions".to_string()),
                musicbrainz_id: Some("123-123-123-123".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist(txn, &Artist {
                name: Some("Wyld Stallions".to_string()),
                musicbrainz_id: Some("456-456-456-456".to_string()),
                ..Default::default()
            })?;
            Ok(())
        })?;
        dbg!(Artist::list(&library));
        assert_eq!(Artist::list(&library).len(), 2);
        Ok(())
    }

    #[test]
    fn test_merge_artist_release_group() -> anyhow::Result<()> {
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            let artist = librarian::merge_artist(txn, &Artist {
                name: Some("Wyld Stallions".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist_release_group(txn, &artist, &&ReleaseGroup {
                title: Some("Awesome Rock Songs".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist_release_group(txn, &artist, &ReleaseGroup {
                title: Some("Awesome Rock Songs".to_string()),
                musicbrainz_id: Some("456-456-456-456".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist_release_group(txn, &artist, &ReleaseGroup {
                title: Some("Badass Metal Songs".to_string()),
                ..Default::default()
            })?;
            Ok(())
        })?;
        assert_eq!(ReleaseGroup::list(&library).len(), 2);
        Ok(())
    }

    #[test]
    fn test_merge_release_group_release() -> anyhow::Result<()> {
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            let artist = librarian::merge_artist(txn, &Artist {
                name: Some("Wyld Stallions".to_string()),
                ..Default::default()
            })?;
            let release_group = librarian::merge_artist_release_group(txn, &artist, &ReleaseGroup {
                title: Some("Awesome Rock Songs".to_string()),
                ..Default::default()
            })?;
            librarian::merge_release_group_release(txn, &release_group, &Release {
                title: Some("Awesome Rock Songs".to_string()),
                ..Default::default()
            })?;
            librarian::merge_release_group_release(txn, &release_group, &Release {
                title: Some("Awesome Rock Songs".to_string()),
                musicbrainz_id: Some("888-888-888-888".to_string()),
                ..Default::default()
            })?;
            librarian::merge_release_group_release(txn, &release_group, &Release {
                title: Some("Awesome Rock Songs".to_string()),
                musicbrainz_id: Some("888-888-888-888".to_string()),
                country: Some("us".to_string()),
                ..Default::default()
            })?;
            librarian::merge_release_group_release(txn, &release_group, &Release {
                title: Some("Awesome Rock Songs".to_string()),
                country: Some("it".to_string()),
                ..Default::default()
            })?;
            Ok(())
        })?;
        assert_eq!(Release::list(&library).len(), 2);
        Ok(())
    }

    #[test]
    fn test_resolve_artist_release() -> anyhow::Result<()> {
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            let artist = librarian::merge_artist(txn, &Artist {
                name: Some("Test Artist".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist_release(txn, &artist, &Release {
                title: Some("Test Album".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist_release(txn, &artist, &Release {
                title: Some("Test Album".to_string()),
                country: Some("us".to_string()),
                ..Default::default()
            })?;
            librarian::merge_artist_release(txn, &artist, &Release {
                title: Some("Another Album".to_string()),
                ..Default::default()
            })?;
            Ok(())
        })?;
        assert_eq!(Release::list(&library).len(), 2);
        Ok(())
    }

    #[test]
    fn test_resolve_release_track() -> anyhow::Result<()> {
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            let artist = librarian::merge_artist(txn, &Artist {
                name: Some("Test Artist".to_string()),
                ..Default::default()
            })?;
            let release = librarian::merge_artist_release(txn, &artist, &Release {
                title: Some("Test Album".to_string()),
                ..Default::default()
            })?;
            librarian::merge_release_track(txn, &release, &Track {
                title: Some("Song One".to_string()),
                position: Some(1),
                ..Default::default()
            })?;
            librarian::merge_release_track(txn, &release, &Track {
                title: Some("Song One".to_string()),
                position: Some(1),
                length_ms: Some(30000),
                ..Default::default()
            })?;
            librarian::merge_release_track(txn, &release, &Track {
                title: Some("Song Two".to_string()),
                position: Some(2),
                ..Default::default()
            })?;
            Ok(())
        })?;
        assert_eq!(Track::list(&library).len(), 2);
        Ok(())
    }

    #[test]
    fn test_artist_extend_only() -> anyhow::Result<()> {
        let library = Library::open_memory();
        let (artist1, artist2) = library.db.transaction(|txn| {
            let artist1 = librarian::merge_artist(txn, &Artist {
                name: Some("Metallica".to_string()),
                ..Default::default()
            })?;
            let artist2 = librarian::merge_artist(txn, &Artist {
                name: Some("Metallica".to_string()),
                country: Some("US".to_string()),
                ..Default::default()
            })?;
            Ok((artist1, artist2))
        })?;
        assert_eq!(artist1.id, artist2.id);
        assert_eq!(artist2.country, Some("US".to_string()));
        Ok(())
    }

    #[test]
    fn test_resolve_genre() -> anyhow::Result<()> {
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            librarian::merge_genre(txn, &Genre {
                name: Some("metal".to_string()),
                ..Default::default()
            })?;
            librarian::merge_genre(txn, &Genre {
                name: Some("Metal".to_string()),
                ..Default::default()
            })?;
            librarian::merge_genre(txn, &Genre {
                name: Some("METAL".to_string()),
                disambiguation: Some("metal! yea!".to_string()),
                musicbrainz_id: Some("123-123-123-123".to_string()),
                ..Default::default()
            })?;
            librarian::merge_genre(txn, &Genre {
                name: Some("metal".to_string()),
                ..Default::default()
            })?;
            Ok(())
        })?;
        assert_eq!(Genre::list(&library).len(), 1);
        Ok(())
    }   

    #[test]
    fn test_match_artist_musicbrainz_id() -> anyhow::Result<()> {
        // country
        let library = Library::open_memory();
        library.db.transaction(|txn| {
            let artist1 = merge_artist(txn, &Artist {
                name: Some("The Napkin".to_string()),
                ..Default::default()
            })?;
            let artist2 = merge_artist(txn, &Artist {
                name: Some("The Napkin".to_string()),
                musicbrainz_id: Some("The Napkin".to_string()),
                ..Default::default()
            })?;   
            let artist3 = merge_artist(txn, &Artist {
                name: Some("The Napkin".to_string()),
                ..Default::default()
            })?;
            assert_eq!(artist1.id, artist2.id);
            assert_eq!(artist2.id, artist3.id);
            Ok(())
        })?;
        Ok(())
    }
}
