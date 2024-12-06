use dimple_core_macro::ModelSupport;

// https://musicbrainz.org/doc/Genre
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Genre {
    pub key: Option<String>,
    pub name: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub liked: bool,
    // pub known_ids: KnownIds,
    // pub links: HashSet<String>,
}

// impl Hash for Genre {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.key.hash(state);
//         self.name.hash(state);
//         self.known_ids.hash(state);
//         self.disambiguation.hash(state);
//         self.summary.hash(state);
//         // self.links.hash(state);
//     }
// }


#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Genre, Diff}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:d83dc0e1-c60a-4267-a9b2-31fcc7ae44bc?mode=memory&cache=shared");
        let mut model = library.save(&Genre::default());
        assert!(model.key.is_some());
        assert!(model.name.is_none());
        model.name = Some("Name".to_string());
        let model = library.save(&model);
        let model: Genre = library.get(&model.key.unwrap()).unwrap();
        assert!(model.name == Some("Name".to_string()));
    }

    #[test]
    fn diff() {
        let a = Genre::default();
        let b = Genre {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            disambiguation: Some("disambiguation".to_string()),
            summary: Some("summary".to_string()),
            liked: true,
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 5);
        dbg!(&diff);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("name".to_string()));
        assert!(diff[2].field == Some("disambiguation".to_string()));
        assert!(diff[3].field == Some("summary".to_string()));
        assert!(diff[4].field == Some("liked".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Genre::default();
        let b = Genre {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            disambiguation: Some("disambiguation".to_string()),
            summary: Some("summary".to_string()),
            liked: true,
        };
        let diff = a.diff(&b);
        let mut c = Genre::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}