use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Dimage, Genre, Link, Release};

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Artist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub country: Option<String>,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,
}

impl Artist {
    pub fn genres(&self, library: &Library) -> Vec<Genre> {
        library.query("
            SELECT g.* FROM GenreRef gr 
            JOIN Genre g ON (g.id = gr.genre_id) 
            WHERE gr.model_id = ?1
        ", (self.id.clone().unwrap(),))
    }

    pub fn links(&self, library: &Library) -> Vec<Link> {
        library.query("
            SELECT l.* FROM LinkRef lr 
            JOIN Link l ON (l.id = lr.link_id) 
            WHERE lr.model_id = ?1
        ", (self.id.clone().unwrap(),))
    }

    pub fn releases(&self, library: &Library) -> Vec<Release> {
        let sql = "
            SELECT Release.* FROM Release
            LEFT JOIN ArtistRef ON (ArtistRef.model_id = Release.id)
            WHERE ArtistRef.artist_id = ?1
            ORDER BY date ASC, title ASC
        ";
        library.query(sql, (self.id.clone(),))
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
    use crate::{librarian::{self, ArtistMetadata}, library::Library, model::{Artist, ArtistRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Artist::default());
        assert!(model.id.is_some());
        assert!(model.name.is_none());
        model.name = Some("Name".to_string());
        let model = library.save(&model);
        let model: Artist = library.get(&model.id.unwrap()).unwrap();
        assert!(model.name == Some("Name".to_string()));
    }
}
