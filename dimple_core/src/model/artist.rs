use dimple_core_macro::ModelSupport;

use crate::library::Library;

use super::Genre;

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
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
}

impl Artist {
    pub fn genres(&self, library: &Library) -> Vec<Genre> {
        library.query("
            SELECT g.* FROM GenreRef gr 
            JOIN Genre g ON (g.key = gr.genre_key) 
            WHERE gr.model_key = ?1
        ", (self.key.clone().unwrap(),))
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Diff, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:59eec92b-6e8e-4839-9eb5-89142890a6a2?mode=memory&cache=shared");
        let mut model = library.save(&Artist::default());
        assert!(model.key.is_some());
        assert!(model.name.is_none());
        model.name = Some("Name".to_string());
        let model = library.save(&model);
        let model: Artist = library.get(&model.key.unwrap()).unwrap();
        assert!(model.name == Some("Name".to_string()));
    }
}