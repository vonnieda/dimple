use crate::library::Library;

use super::{Genre, LibraryModel};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenreRef {
    pub model_key: String,
    pub genre_key: String,
}

impl GenreRef {
    pub fn attach(library: &Library, genre: &Genre, model: &impl LibraryModel) {
        let _ = library.conn().execute(
            "INSERT INTO GenreRef (genre_key, model_key) VALUES (?, ?)", 
            (genre.key.clone(), model.key()));
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Genre, GenreRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let genre = library.save(&Genre {
            name: Some("Test".to_string()),
            ..Default::default()
        });
        let track = library.save(&Track::default());
        GenreRef::attach(&library, &genre, &track);
        assert!(track.genres(&library).len() == 1);
    }
}

