use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Artist, Dimage, Genre, Link, ModelBasics as _, Release};

// // https://musicbrainz.org/doc/Track
// // https://musicbrainz.org/ws/2/release/4d3ce256-ea71-44c5-8ce9-deb8f1e7dce4?inc=aliases%2Bartist-credits%2Blabels%2Bdiscids%2Brecordings&fmt=json
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<String>,
    pub title: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub release_id: Option<String>,

    pub position: Option<u32>,
    pub length_ms: Option<u64>,
    pub lyrics: Option<String>,
    // pub instrumental: Option<bool>;
    // LRC format (https://en.wikipedia.org/wiki/LRC_(file_format))
    pub synchronized_lyrics: Option<String>,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,

    // Have decided not to create a separate model for Medium for now, so
    // denormalized that data into Track.
    pub media_track_count: Option<u32>,
    pub media_position: Option<u32>,
    pub media_title: Option<String>,
    pub media_format: Option<String>,
}

impl Track {
    pub fn release(&self, library: &Library) -> Option<Release> {
        self.release_id.clone().and_then(|id| library.get(&id))
    }

    pub fn album_name(&self, library: &Library) -> Option<String> {
        self.release(library).and_then(|r| r.title)
    }

    pub fn artist(&self, library: &Library) -> Option<Artist> {
        self.artists(library).get(0).cloned()
    }

    pub fn artist_name(&self, library: &Library) -> Option<String> {
        self.artist(library).and_then(|a| a.name)
    }

    /// TODO this should return the artists in order, with the primary being
    /// first. I'm not exactly sure how to indicate primary yet.
    pub fn artists(&self, library: &Library) -> Vec<Artist> {
        self.id.as_ref().map(|id| {
            library.query("
                SELECT a.* FROM ArtistRef ar 
                JOIN Artist a ON (a.id = ar.artist_id) 
                WHERE ar.model_id = ?1
                ORDER BY ar.rowid ASC
            ", (id,))
        }).unwrap_or_default()
    }

    pub fn genres(&self, library: &Library) -> Vec<Genre> {
        self.id.as_ref().map(|id| {
            library.query("
                SELECT g.* FROM GenreRef gr 
                JOIN Genre g ON (g.id = gr.genre_id) 
                WHERE gr.model_id = ?1
                ORDER BY g.name ASC
            ", (id,))
        }).unwrap_or_default()
    }

    pub fn links(&self, library: &Library) -> Vec<Link> {
        self.id.as_ref().map(|id| {
            library.query("
                SELECT l.* FROM LinkRef lr 
                JOIN Link l ON (l.id = lr.link_id) 
                WHERE lr.model_id = ?1
                ORDER BY l.url ASC
            ", (id,))
        }).unwrap_or_default()
    }

    pub fn images(&self, library: &Library) -> Vec<Dimage> {
        library.query("
            SELECT d.* FROM DimageRef dr 
            JOIN Dimage d ON (d.id = dr.dimage_id) 
            WHERE dr.model_id = ?1
        ", (self.id.clone().unwrap(),))
    }
}

#[cfg(test)]
mod tests {
    use std::{hash::DefaultHasher};

    use crate::{library::Library, model::{Artist, ArtistRef, Genre, GenreRef}};

    use super::{Track};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Track::default());
    }

    #[test]
    fn artists() {
        let library = Library::open_memory();
        let track = library.save(&Track {
            title: Some("Lucy".to_string()),
            ..Default::default()
        }).unwrap();

        let _ = library.db.transaction(|t| {
            let artist = t.save(&Artist {
                name: Some("Metallica".to_string()),
                ..Default::default()
            })?;
            ArtistRef::attach(t, &artist, &track.id)?;
            let artist = t.save(&Artist {
                name: Some("Lou Reed".to_string()),
                ..Default::default()
            })?;
            ArtistRef::attach(t, &artist, &track.id)
        });

        // dbg!(track.artists(&library));
    }

    #[test]
    fn genres() {
        let library = Library::open_memory();
        let death_metal = library.save(&Genre {
            name: Some("death metal".to_string()),
            ..Default::default()
        }).unwrap();
        let heavy_metal = library.save(&Genre {
            name: Some("heavy metal".to_string()),
            ..Default::default()
        }).unwrap();
        let rock = library.save(&Genre {
            name: Some("rock".to_string()),
            ..Default::default()
        }).unwrap();
        let _smooth_jazz = library.save(&Genre {
            name: Some("smooth jazz".to_string()),
            ..Default::default()
        }).unwrap();
        let _jazz = library.save(&Genre {
            name: Some("jazz".to_string()),
            ..Default::default()
        }).unwrap();

        let track = library.save(&Track {
            title: Some("Lucy".to_string()),
            ..Default::default()
        }).unwrap();
        let _ = library.db.transaction(|t| GenreRef::attach(t, &heavy_metal, &track.id));
        let _ = library.db.transaction(|t| GenreRef::attach(t, &rock, &track.id));

        let artist = library.save(&Artist {
            name: Some("Metallica".to_string()),
            ..Default::default()
        }).unwrap();
        let _ = library.db.transaction(|t| GenreRef::attach(t, &rock, &artist.id));
        let _ = library.db.transaction(|t| GenreRef::attach(t, &heavy_metal, &artist.id));
        let _ = library.db.transaction(|t| GenreRef::attach(t, &death_metal, &artist.id));

        assert!(artist.genres(&library).len() == 3);
        assert!(track.genres(&library).len() == 2);
        assert!(library.list::<Genre>().len() == 5); 
    }
}
