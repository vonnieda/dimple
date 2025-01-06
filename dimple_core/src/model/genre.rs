use dimple_core_macro::ModelSupport;

// https://musicbrainz.org/doc/Genre
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
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
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Genre, Diff}};

    #[test]
    fn library_crud() {
        let library = Library::open_temporary();
        let model = library.save(&Genre {
            name: Some("The Meat Puppets".to_string()),
            ..Default::default()
        });
        assert!(model.key.is_some());
        assert!(model.name == Some("The Meat Puppets".to_string()));
    }
}