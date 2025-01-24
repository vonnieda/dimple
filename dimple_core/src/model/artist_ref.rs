use crate::library::Library;

use super::{Artist, LibraryModel};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArtistRef {
    pub model_key: String,
    pub artist_key: String,
}

impl ArtistRef {
    pub fn attach(library: &Library, artist: &Artist, model: &impl LibraryModel) {
        let _ = library.conn().execute(
            "INSERT INTO ArtistRef (artist_key, model_key) VALUES (?, ?)", 
            (artist.key.clone(), model.key()));
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let artist = library.save(&Artist::default());
        let track = library.save(&Track::default());
        ArtistRef::attach(&library, &artist, &track);
        assert!(track.artists(&library).len() == 1);
    }
}

