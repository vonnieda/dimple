use dimple_core_macro::{model_ignore, ModelSupport};

use crate::library::Library;

use super::{Dimage, Genre, Link, Release};

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, ModelSupport)]
pub struct Artist {
    pub key: Option<String>,
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

    #[model_ignore]
    pub releases: Vec<Release>,
    #[model_ignore]
    pub genres: Vec<Genre>,
    #[model_ignore]
    pub links: Vec<Link>,
}

impl Artist {
    pub fn genres(&self, library: &Library) -> Vec<Genre> {
        library.query("
            SELECT g.* FROM GenreRef gr 
            JOIN Genre g ON (g.key = gr.genre_key) 
            WHERE gr.model_key = ?1
        ", (self.key.clone().unwrap(),))
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
            LEFT JOIN ArtistRef ON (ArtistRef.model_key = Release.key)
            WHERE ArtistRef.artist_key = ?1
            ORDER BY date ASC, title ASC
        ";
        library.query(sql, (self.key.clone(),))
    }

    pub fn images(&self, library: &Library) -> Vec<Dimage> {
        library.query("
            SELECT d.* FROM DimageRef dr 
            JOIN Dimage d ON (d.key = dr.dimage_key) 
            WHERE dr.model_key = ?1
        ", (self.key.clone().unwrap(),))
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Diff, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Artist::default());
        assert!(model.key.is_some());
        assert!(model.name.is_none());
        model.name = Some("Name".to_string());
        let model = library.save(&model);
        let model: Artist = library.get(&model.key.unwrap()).unwrap();
        assert!(model.name == Some("Name".to_string()));
    }
}