use anyhow::anyhow;
use dimple_db::db::transaction::DbTransaction;
use crate::{crdt_rules::CrdtRules, librarian, library::Library, merge_rules::{MergeError, MergeRules}, model::{Artist, ArtistRef, Dimage, DimageRef, DimpleEntity, Genre, GenreRef, Link, LinkRef, Release, ReleaseGroup, ReleaseGroupSecondaryType, ReleaseGroupSecondaryTypeRef, Track}, plugins::plugins::Plugins};

/// TODO currently trying two different methods in the match portion of this
/// module: fts5, and direct query. I think fts5 is overall better because we
/// can score it and has weights already built in. direct query primarily
/// solves the issue I was having where I could not include other clauses
/// in the where clause. 
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
        DimpleEntity::Track(track) => {
            for metadata in plugins.track_metadata(library, track) {
                library.db.transaction(|t| {
                    merge_track_metadata(t, track, &metadata)
                }).unwrap();
            }
        },
        // DimpleEntity::Genre(genre) => {
        //     if let Some(metadata) = plugins.metadata(library, &genre.clone()) {
        //         library.save(&CrdtRules::merge(genre, metadata));
        //     }
        // },
        // DimpleEntity::Release(release) => {
        //     for metadata in plugins.release_metadata(library, release) {
        //         let _ = library.db.transaction(|t| 
        //             librarian::merge_release_metadata(t, &metadata, Some(release.clone())));
        //     }
        // },
        _ => todo!()
    }
}

pub fn merge_artist(txn: &DbTransaction, artist: &Artist) -> Result<Artist, anyhow::Error> {
    let mut query_fragments = vec![];
    if let (Some(name), Some(disambiguation)) = (&artist.name, &artist.disambiguation) {
        query_fragments.push(format!("(name:{} AND disambiguation:{})", quote_fts5(name), quote_fts5(disambiguation)));
    }
    else if let Some(name) = &artist.name {
        query_fragments.push(format!("name:{}", quote_fts5(name)));
    }
    if let Some(discogs_id) = &artist.discogs_id {
        query_fragments.push(format!("discogs_id:{}", quote_fts5(discogs_id)));
    }
    if let Some(lastfm_id) = &artist.lastfm_id {
        query_fragments.push(format!("lastfm_id:{}", quote_fts5(lastfm_id)));
    }
    if let Some(musicbrainz_id) = &artist.musicbrainz_id {
        query_fragments.push(format!("musicbrainz_id:{}", quote_fts5(musicbrainz_id)));
    }
    if let Some(spotify_id) = &artist.spotify_id {
        query_fragments.push(format!("spotify_id:{}", quote_fts5(spotify_id)));
    }
    let query = query_fragments.join(" OR ");
    let sql = format!("
        SELECT Artist.* 
        FROM ArtistFts
        JOIN Artist ON (ArtistFts.rowid = Artist.rowid)
        WHERE ArtistFts MATCH ?
        ORDER BY bm25(ArtistFts);
    ");

    let candidates: Vec<Artist> = txn.query(&sql, (query,))?;
    for candidate in candidates.iter() {
        if let Ok(merged) = Artist::try_merge(candidate, artist) {
            if merged != *artist {
                return txn.save(&merged)
            }
        }
    }
    txn.save(&artist)
}

pub fn merge_artist_metadata(txn: &DbTransaction, artist: &Artist, metadata: &ArtistMetadata) -> Result<Artist, anyhow::Error> {
    let mut merged = CrdtRules::merge(artist.clone(), metadata.artist.clone());
    if merged != *artist {
        merged = txn.save(&merged)?;
    }
    merge_entity_genres(txn, &merged.id, &metadata.genres)?;
    merge_entity_links(txn, &merged.id, &metadata.links)?;
    merge_entity_images(txn, &merged.id, &metadata.images)?;
    Ok(merged)
}

pub fn merge_artist_release_group(txn: &DbTransaction, artist: &Artist, 
    release_group: &ReleaseGroup) 
    -> Result<ReleaseGroup, anyhow::Error> {
    
    let sql = format!("
        SELECT ReleaseGroup.* 
        FROM ReleaseGroup
        JOIN ArtistRef ON (ReleaseGroup.id = ArtistRef.model_id)
        WHERE ArtistRef.artist_id = ?1
        AND (
            (ReleaseGroup.title IS NOT NULL AND ReleaseGroup.title COLLATE NOCASE = ?2 AND ((ReleaseGroup.disambiguation IS NULL AND ?3 IS NULL) OR (ReleaseGroup.disambiguation COLLATE NOCASE = ?3)))
            OR (ReleaseGroup.discogs_id IS NOT NULL AND ReleaseGroup.discogs_id = ?4)
            OR (ReleaseGroup.lastfm_id IS NOT NULL AND ReleaseGroup.lastfm_id = ?5)
            OR (ReleaseGroup.musicbrainz_id IS NOT NULL AND ReleaseGroup.musicbrainz_id = ?6)
            OR (ReleaseGroup.spotify_id IS NOT NULL AND ReleaseGroup.spotify_id = ?7)
        )
    ");

    let candidates: Vec<ReleaseGroup> = txn.query(&sql, (&artist.id, 
        &release_group.title, 
        &release_group.disambiguation, 
        &release_group.discogs_id, 
        &release_group.lastfm_id, 
        &release_group.musicbrainz_id,
        &release_group.spotify_id,)
    )?;
    for candidate in candidates.iter() {
        if let Ok(merged) = ReleaseGroup::try_merge(candidate, release_group) {
            if merged != *release_group {
                return txn.save(&merged)
            }
        }
    }
    let merged = txn.save(release_group)?;
    ArtistRef::attach(txn, artist, &merged.id)?;
    return Ok(merged)
}

pub fn merge_release_group(txn: &DbTransaction, release_group: &ReleaseGroup) -> anyhow::Result<ReleaseGroup> {
    let sql = format!("
        SELECT ReleaseGroup.* 
        FROM ReleaseGroup
        WHERE (ReleaseGroup.title IS NOT NULL AND ReleaseGroup.title COLLATE NOCASE = ?1 AND ((ReleaseGroup.disambiguation IS NULL AND ?2 IS NULL) OR (ReleaseGroup.disambiguation COLLATE NOCASE = ?2)))
        AND (
            (ReleaseGroup.discogs_id IS NOT NULL AND ReleaseGroup.discogs_id = ?3)
            OR (ReleaseGroup.lastfm_id IS NOT NULL AND ReleaseGroup.lastfm_id = ?4)
            OR (ReleaseGroup.musicbrainz_id IS NOT NULL AND ReleaseGroup.musicbrainz_id = ?5)
            OR (ReleaseGroup.spotify_id IS NOT NULL AND ReleaseGroup.spotify_id = ?6)
        )
    ");

    let candidates: Vec<ReleaseGroup> = txn.query(&sql, 
        (&release_group.title, 
        &release_group.disambiguation, 
        &release_group.discogs_id, 
        &release_group.lastfm_id, 
        &release_group.musicbrainz_id,
        &release_group.spotify_id,)
    )?;
    for candidate in candidates.iter() {
        if let Ok(merged) = ReleaseGroup::try_merge(release_group, candidate) {
            if merged != *release_group {
                return txn.save(&merged)
            }
        }
    }
    txn.save(&release_group)
}

pub fn merge_release_group_metadata(txn: &DbTransaction, release_group: &ReleaseGroup, metadata: &ReleaseGroupMetadata) -> Result<ReleaseGroup, anyhow::Error> {
    let mut merged = CrdtRules::merge(release_group.clone(), metadata.release_group.clone());
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

pub fn merge_release_group_release(txn: &DbTransaction, release_group: &ReleaseGroup, release: &Release) -> Result<Release, anyhow::Error> {
    let sql = format!("
        SELECT Release.* 
        FROM Release
        WHERE Release.release_group_id = ?1
        AND (
            (Release.title IS NOT NULL AND Release.title COLLATE NOCASE = ?2 AND ((Release.disambiguation IS NULL AND ?3 IS NULL) OR (Release.disambiguation COLLATE NOCASE = ?3)))
            OR (Release.discogs_id IS NOT NULL AND Release.discogs_id = ?4)
            OR (Release.lastfm_id IS NOT NULL AND Release.lastfm_id = ?5)
            OR (Release.musicbrainz_id IS NOT NULL AND Release.musicbrainz_id = ?6)
            OR (Release.spotify_id IS NOT NULL AND Release.spotify_id = ?7)
        )
    ");

    let candidates: Vec<Release> = txn.query(&sql, (&release_group.id, 
        &release.title, 
        &release.disambiguation, 
        &release.discogs_id, 
        &release.lastfm_id, 
        &release.musicbrainz_id,
        &release.spotify_id,)
    )?;
    for candidate in candidates.iter() {
        if let Ok(merged) = Release::try_merge(candidate, release) {
            if merged != *release {
                return txn.save(&merged)
            }
        }
    }
    let mut release = release.clone();
    release.release_group_id = release_group.id.clone();
    txn.save(&release)
}

pub fn merge_release_metadata(txn: &DbTransaction, release: &Release, metadata: &ReleaseMetadata) -> Result<Release, anyhow::Error> {
    let mut merged = CrdtRules::merge(release.clone(), metadata.release.clone());
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

pub fn merge_release_track(txn: &DbTransaction, release: &Release, track: &Track) -> Result<Track, anyhow::Error> {
     let sql = format!("
        SELECT Track.* 
        FROM Track
        WHERE Track.release_id = ?1
        AND (
            (Track.title IS NOT NULL AND Track.title COLLATE NOCASE = ?2 AND ((Track.disambiguation IS NULL AND ?3 IS NULL) OR (Track.disambiguation COLLATE NOCASE = ?3)))
            OR (Track.discogs_id IS NOT NULL AND Track.discogs_id = ?4)
            OR (Track.lastfm_id IS NOT NULL AND Track.lastfm_id = ?5)
            OR (Track.musicbrainz_id IS NOT NULL AND Track.musicbrainz_id = ?6)
            OR (Track.spotify_id IS NOT NULL AND Track.spotify_id = ?7)
        )
    ");

    let candidates: Vec<Track> = txn.query(&sql, (&release.id, 
        &release.title, 
        &release.disambiguation, 
        &release.discogs_id, 
        &release.lastfm_id, 
        &release.musicbrainz_id,
        &release.spotify_id,)
    )?;
    for candidate in candidates.iter() {
        if let Ok(merged) = Track::try_merge(candidate, track) {
            if merged != *track {
                return txn.save(&merged)
            }
        }
    }
    let mut track = track.clone();
    track.release_id = release.id.clone();
    txn.save(&track)
}


pub fn merge_track_metadata(txn: &DbTransaction, track: &Track, metadata: &TrackMetadata) -> Result<Track, anyhow::Error> {
    let mut merged = CrdtRules::merge(track.clone(), metadata.track.clone());
    if merged != *track {
        merged = txn.save(&merged)?;
    }
    merge_entity_artists(txn, &merged.id, &metadata.artists)?;
    merge_entity_genres(txn, &merged.id, &metadata.genres)?;
    merge_entity_links(txn, &merged.id, &metadata.links)?;
    merge_entity_images(txn, &merged.id, &metadata.images)?;
    Ok(merged)
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
        let dimage = merge_image(txn, dimage)?;
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
    let matched = match_link(txn, link)?.unwrap_or_default();
    let merged = CrdtRules::merge(matched.clone(), link.clone());
    if matched == merged {
        return Ok(matched)
    }
    txn.save(&merged)
}

fn merge_image(txn: &DbTransaction, dimage: &Dimage) -> Result<Dimage, anyhow::Error> {
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

fn match_genre(txn: &DbTransaction, genre: &Genre) -> Result<Option<Genre>, anyhow::Error> {
    let results = txn.query("
        SELECT Genre.* 
        FROM Genre 
        WHERE (Genre.name IS NOT NULL AND Genre.name COLLATE NOCASE = ?1 AND ((Genre.disambiguation IS NULL AND ?2 IS NULL) OR (Genre.disambiguation COLLATE NOCASE = ?2)))
        OR (Genre.discogs_id IS NOT NULL AND Genre.discogs_id = ?3)
        OR (Genre.lastfm_id IS NOT NULL AND Genre.lastfm_id = ?4)
        OR (Genre.musicbrainz_id IS NOT NULL AND Genre.musicbrainz_id = ?5)
        OR (Genre.spotify_id IS NOT NULL AND Genre.spotify_id = ?6)
        ", (&genre.name, &genre.disambiguation, 
            &genre.discogs_id, &genre.lastfm_id, &genre.musicbrainz_id, 
            &genre.spotify_id,)
    )?;
    Ok(results.into_iter().next())
}

fn match_link(txn: &DbTransaction, link: &Link) -> Result<Option<Link>, anyhow::Error> {
    let results = txn.query("
        SELECT Link.* 
        FROM Link 
        WHERE (Link.url = ?1)
        ", (&link.url,))?;
    Ok(results.into_iter().next())
}

fn match_dimage(txn: &DbTransaction, dimage: &Dimage) -> Result<Option<Dimage>, anyhow::Error> {
    let results = txn.query("
        SELECT Dimage.* 
        FROM Dimage 
        WHERE (Dimage.sha256 = ?1)
        ", (&dimage.sha256,))?;
    Ok(results.into_iter().next())
}

fn quote_fts5(s: &String) -> String {
    format!("\"{}\"", s.replace("\"", "\"\""))
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
    pub tracks: Vec<TrackMetadata>,
    pub images: Vec<Dimage>,
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
pub struct SearchResults {
    pub tracks: Vec<TrackMetadata>,
    pub artists: Vec<ArtistMetadata>,
    pub releases: Vec<ReleaseMetadata>,
    pub release_groups: Vec<ReleaseGroupMetadata>,
    pub genres: Vec<Genre>,
}

mod tests {
    use crate::{librarian::{self, ArtistMetadata, ReleaseMetadata}, library::Library, model::{Artist, ModelBasics, Release, ReleaseGroup, Track}, plugins::plugins::Plugins};

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
            })
        })?;
        assert_eq!(Artist::list(&library).len(), 1);
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

    // #[test]
    // fn create_or_update_artist_metadata() {
    //     let library = Library::open_memory();
    //     let (artist1, artist2, artist3, artist4) = library.db.transaction(|txn| {
    //         let artist1 = librarian::create_or_update_artist_metadata(txn, &ArtistMetadata {
    //             artist: Artist {
    //                 name: Some("Something Cool".to_string()),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         })?;
    //         let artist2 = librarian::create_or_update_artist_metadata(txn, &ArtistMetadata {
    //             artist: Artist {
    //                 name: Some("Something Cool".to_string()),
    //                 musicbrainz_id: Some("4563463".to_string()),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         })?;
    //         let artist3 = librarian::create_or_update_artist_metadata(txn, &ArtistMetadata {
    //             artist: Artist {
    //                 name: Some("Something Cool".to_string()),
    //                 disambiguation: Some("the other one".to_string()),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         })?;
    //         let artist4 = librarian::create_or_update_artist_metadata(txn, &ArtistMetadata {
    //             artist: Artist {
    //                 name: Some("Something Cool".to_string()),
    //                 disambiguation: Some("the other one".to_string()),
    //                 musicbrainz_id: Some("123123".to_string()),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         })?;
    //         Ok((artist1, artist2, artist3, artist4))
    //     }).unwrap();
    //     assert_eq!(artist1.id, artist2.id);        
    //     assert_ne!(artist2.id, artist3.id);
    //     assert_eq!(artist3.id, artist4.id);
    // }
}
