use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::Genre;

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Link {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let model = library.save(&Artist::default()).unwrap();
        assert!(model.id.is_some());
    }
}