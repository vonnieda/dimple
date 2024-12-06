use dimple_core_macro::ModelSupport;

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub liked: bool,

    pub country: Option<String>,

    // pub known_ids: KnownIds,
    // pub links: HashSet<String>,
    // pub genres: Vec<Genre>,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, Diff}};

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

    #[test]
    fn diff() {
        let a = Artist::default();
        let b = Artist {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            disambiguation: Some("disambiguation".to_string()),
            summary: Some("summary".to_string()),
            liked: true,
            country: Some("country".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 6);
        dbg!(&diff);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("name".to_string()));
        assert!(diff[2].field == Some("disambiguation".to_string()));
        assert!(diff[3].field == Some("summary".to_string()));
        assert!(diff[4].field == Some("liked".to_string()));
        assert!(diff[5].field == Some("country".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Artist::default();
        let b = Artist {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            disambiguation: Some("disambiguation".to_string()),
            summary: Some("summary".to_string()),
            liked: true,
            country: Some("country".to_string()),
        };
        let diff = a.diff(&b);
        let mut c = Artist::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}