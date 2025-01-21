use dimple_core_macro::ModelSupport;

use crate::library::Library;

use super::{Link, Release};

// https://musicbrainz.org/doc/Genre
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, ModelSupport)]
pub struct Genre {
    pub key: Option<String>,
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
            JOIN Link l ON (l.key = lr.link_key) 
            WHERE lr.model_key = ?1
        ", (self.key.clone().unwrap(),))
    }

    pub fn releases(&self, library: &Library) -> Vec<Release> {
        let sql = "
            SELECT Release.* FROM Release
            LEFT JOIN GenreRef ON (GenreRef.model_key = Release.key)
            WHERE GenreRef.genre_key = ?1
            ORDER BY title ASC
        ";
        library.query(sql, (self.key.clone(),))
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Genre, Diff}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let model = library.save(&Genre {
            name: Some("The Meat Puppets".to_string()),
            ..Default::default()
        });
        assert!(model.key.is_some());
        assert!(model.name == Some("The Meat Puppets".to_string()));
    }
}