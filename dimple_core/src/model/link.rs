use dimple_core_macro::ModelSupport;

use crate::library::Library;

use super::Genre;

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Link {
    pub key: Option<String>,
    pub name: Option<String>,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Diff, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:59eec92b-6e8e-4839-9eb5-89142890a6a2?mode=memory&cache=shared");
        let model = library.save(&Artist::default());
        assert!(model.key.is_some());
    }
}