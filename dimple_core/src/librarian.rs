use dimple_db::db::Entity;
use image::DynamicImage;

use crate::{librarian, library::Library, merge::CrdtRules, model::{Artist, ArtistRef, Dimage, DimageRef, DimpleEntity, Genre, GenreRef, Link, LinkRef, ModelBasics as _, Release, Track}, plugins::plugins::Plugins};

#[derive(Clone)]
pub struct Librarian {
    library: Library,
    plugins: Plugins,
}

impl Librarian {
    pub fn new(library: &Library, plugins: &Plugins) -> Self {
        Self {
            library: library.clone(),
            plugins: plugins.clone(),
        }
    }

    pub fn image(&self, model: &DimpleEntity) -> Option<DynamicImage> {
        if let Some(image) = self.library.image(model) {
            return Some(image)
        }
    
        for dimage in self.plugins.image(&self.library, model) {
            let dimage = merge_image(&self.library, &dimage);
            DimageRef::attach(&self.library, &dimage, &Some(model.id()));
        }
    
        self.library.image(model)
    }    

    pub fn search(&self, query: &str) -> SearchResults {
        let plugin_results = self.plugins.search(&self.library, query);
    
        for result in plugin_results {
            for artist in result.artists {
                librarian::merge_artist(&self.library, &artist);
            }
        }
    
        let query = format!("%{}%", query);
        let artists = Artist::query(&self.library, 
            "SELECT * FROM Artist WHERE name LIKE ?1 LIMIT 25", (&query,));
        let releases = Release::query(&self.library, 
            "SELECT * FROM Release WHERE title LIKE ?1 LIMIT 25", (&query,));
        let genres = Genre::query(&self.library, 
            "SELECT * FROM Genre WHERE name LIKE ?1 LIMIT 25", (&query,));
        let tracks = Track::query(&self.library, 
            "SELECT * FROM Track WHERE title LIKE ?1 LIMIT 25", (&query,));    
    
        SearchResults { 
            artists, 
            releases, 
            genres, 
            tracks, 
            ..Default::default()
        }
    }    
}

pub fn refresh_metadata(library: &Library, plugins: &Plugins, model: &DimpleEntity) {
    match model {
        DimpleEntity::Artist(artist) => {
            for metadata in plugins.artist_metadata(library, &artist) {
                librarian::merge_artist_metadata(library, &metadata, Some(artist.clone()));
            }
        },
        DimpleEntity::Track(track) => {
            for metadata in plugins.track_metadata(library, &track) {
                librarian::merge_track_metadata(library, &metadata, Some(track.clone()));
            }
        },
        DimpleEntity::Genre(genre) => {
            // if let Some(metadata) = plugins.metadata(library, &genre.clone()) {
            //     library.save(&CrdtRules::merge(genre, metadata));
            // }
        },
        DimpleEntity::Release(release) => {
            for metadata in plugins.release_metadata(library, &release) {
                librarian::merge_release_metadata(library, &metadata, Some(release.clone()));
            }
        },
        _ => todo!()
    }
}

// TODO alllll this stuff has to change to take a DbTransaction I think.
pub fn merge_artist(library: &Library, artist: &Artist) -> Artist {
    let matched = match_artist(library, artist).unwrap_or_default();
    let merged = CrdtRules::merge(matched, artist.clone());
    merged.save(library)
}

pub fn merge_artist_metadata(library: &Library, artist: &ArtistMetadata, pre_match: Option<Artist>) -> Artist {
    let matched = pre_match.or_else(|| match_artist(library, &artist.artist)).unwrap_or_default();
    let merged = CrdtRules::merge(matched, artist.artist.clone());
    let merged = merged.save(library);
    merge_genres(library, &artist.genres, &merged.id);
    merge_links(library, &artist.links, &merged.id);
    merge_images(library, &artist.images, &merged.id);
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
    merge_artists(library, &metadata.artists, &merged.id);
    merge_genres(library, &metadata.genres, &merged.id);
    merge_links(library, &metadata.links, &merged.id);
    merge_images(library, &metadata.images, &merged.id);
    merged
}

pub fn merge_track_metadata(library: &Library, metadata: &TrackMetadata, pre_match: Option<Track>) -> Track {
    let matched = pre_match.or_else(|| match_track(library, &metadata)).unwrap_or_default();
    let merged = CrdtRules::merge(matched, metadata.track.clone());
    let mut merged = merged.save(library);
    merge_artists(library, &metadata.artists, &merged.id);
    merge_genres(library, &metadata.genres, &merged.id);
    merge_links(library, &metadata.links, &merged.id);
    merge_images(library, &metadata.images, &merged.id);
    if let Some(release) = metadata.release.clone() {
        let release = merge_release_metadata(library, &release, merged.release(library));
        merged.release_id = release.id;
        merged = merged.save(&library);
    }
    merged
}

pub fn merge_link(library: &Library, link: &Link) -> Link {
    let matched = match_link(library, &link).unwrap_or_default();
    let link = CrdtRules::merge(matched, link.clone());
    link.save(library)
}

pub fn merge_images(library: &Library, images: &[Dimage], model_id: &Option<String>) {
    for dimage in images {
        let dimage = merge_image(library, dimage);
        DimageRef::attach(library, &dimage, model_id);
    }
}

pub fn merge_image(library: &Library, dimage: &Dimage) -> Dimage {
    let matched = match_dimage(library, &dimage).unwrap_or_default();
    let dimage = CrdtRules::merge(matched, dimage.clone());
    dimage.save(library)
}

pub fn merge_links(library: &Library, links: &[Link], model_id: &Option<String>) {
    for link in links {
        let link = merge_link(library, link);
        merge_link_ref(library, &link, model_id);
    }
}

pub fn merge_genre(library: &Library, genre: &Genre) -> Genre {
    let matched = match_genre(library, &genre).unwrap_or_default();
    let genre = CrdtRules::merge(matched, genre.clone());
    genre.save(library)
}

pub fn merge_genres(library: &Library, genres: &[Genre], model_id: &Option<String>) {
    for genre in genres {
        let genre = merge_genre(library, genre);
        merge_genre_ref(library, &genre, model_id);
    }
}

pub fn merge_artists(library: &Library, artists: &[ArtistMetadata], model_id: &Option<String>) {
    for artist in artists {
        let artist = merge_artist_metadata(library, &artist, None);
        merge_artist_ref(library, &artist, model_id);
    }
}

pub fn merge_artist_ref(library: &Library, artist: &Artist, model_id: &Option<String>) {
    ArtistRef::attach(library, artist, model_id);
}

pub fn merge_genre_ref(library: &Library, genre: &Genre, model_id: &Option<String>) {
    GenreRef::attach(library, genre, model_id);
}

pub fn merge_link_ref(library: &Library, link: &Link, model_id: &Option<String>) {
    LinkRef::attach(library, link, model_id);
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
            LEFT JOIN ArtistRef rar ON (rar.model_id = r.id)
            LEFT JOIN Artist ra ON (ra.id = rar.artist_id)
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
                LEFT JOIN Release r ON (r.id = t.release_id)
                LEFT JOIN ArtistRef tar ON (tar.model_id = t.id)
                LEFT JOIN Artist ta ON (ta.id = tar.artist_id)
                LEFT JOIN ArtistRef rar ON (rar.model_id = r.id)
                LEFT JOIN Artist ra ON (ra.id = rar.artist_id)
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
    use std::sync::Arc;

    use anyhow::Result;

    use crate::{librarian::{self, ArtistMetadata, Librarian}, library::Library, model::{Artist, DimpleEntity, ModelBasics}, plugins::{example::ExamplePlugin, fanart_tv::FanartTvPlugin, lrclib::LrclibPlugin, musicbrainz::MusicBrainzPlugin, plugins::Plugins, wikidata::WikidataPlugin}};

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
        assert!(artist1.id != artist3.id);
        assert!(artist1.id == artist2.id);
        assert!(artist3.id == artist4.id);
    }

    #[test]
    #[ignore]
    fn image() -> Result<()> {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        library.notifier.observe(|e| {
            dbg!(e.type_name, e.key);
        });
        let plugins = Plugins::default();
        plugins.add_plugin(Arc::new(MusicBrainzPlugin::default()));
        plugins.add_plugin(Arc::new(WikidataPlugin::default()));
        plugins.add_plugin(Arc::new(LrclibPlugin::default()));
        plugins.add_plugin(Arc::new(FanartTvPlugin::default()));
        plugins.add_plugin(Arc::new(ExamplePlugin::default()));
        let librarian = Librarian::new(&library, &plugins);
        let artist = library.save(&Artist {
            musicbrainz_id: Some("6821bf3f-5d5b-4b0f-8fa4-79d2ab2d9219".to_string()),
            ..Default::default()
        })?;
        let image = librarian.image(&artist.into()).unwrap();
        assert!(image.width() > 0 && image.height() > 0);
        Ok(())
    }

    #[test]
    fn basics() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        plugins.add_plugin(Arc::new(MusicBrainzPlugin::default()));
        plugins.add_plugin(Arc::new(WikidataPlugin::default()));
        plugins.add_plugin(Arc::new(LrclibPlugin::default()));
        plugins.add_plugin(Arc::new(FanartTvPlugin::default()));
        plugins.add_plugin(Arc::new(ExamplePlugin::default()));
        let librarian = Librarian::new(&library, &plugins);

        let results = librarian.search("Black Sabbath");
        let artist = results.artists.get(0).unwrap().clone();
        assert!(artist.musicbrainz_id == Some("5182c1d9-c7d2-4dad-afa0-ccfeada921a8".to_string()));

        // let releases = artist.releases(&library)
    }
}