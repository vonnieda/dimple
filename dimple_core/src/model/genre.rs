use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Artist, Dimage, Link, Release};

// https://musicbrainz.org/doc/Genre
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Genre {
    pub id: Option<String>,
    pub name: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,
}

impl Genre {
    pub fn new(name: &str) -> Self {
        Self { 
            name: Some(name.to_string()),
            ..Default::default()
        }
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
            LEFT JOIN GenreRef ON (GenreRef.model_id = Release.id)
            WHERE GenreRef.genre_id = ?1
            ORDER BY title ASC
        ";
        library.query(sql, (self.id.clone(),))
    }

    pub fn artists(&self, library: &Library) -> Vec<Artist> {
        let sql = "
            SELECT Artist.* FROM Artist
            LEFT JOIN GenreRef ON (GenreRef.model_id= Artist.id)
            WHERE GenreRef.genre_id = ?1
            ORDER BY name ASC
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
    use crate::{library::Library, model::{Genre}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let model = library.save(&Genre {
            name: Some("The Meat Puppets".to_string()),
            ..Default::default()
        }).unwrap();
        assert!(model.id.is_some());
        assert!(model.name == Some("The Meat Puppets".to_string()));
    }
}