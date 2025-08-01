use dimple_db::db::Entity;
use musicbrainz_rs::entity;
use serde::{Deserialize, Serialize};

use crate::{library::Library, model::{track, Track}};

use super::{Artist};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArtistRef {
    pub id: Option<String>,
    pub model_id: String,
    pub artist_id: String,
}

impl ArtistRef {
    pub fn attach(library: &Library, artist: &Artist, model_id: &Option<String>) {
        let _ = library.save(&ArtistRef {
            model_id: model_id.clone().unwrap(),
            artist_id: artist.id.clone().unwrap(),
            ..Default::default()
        });
    }    
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let artist = library.save(&Artist::default()).unwrap();
        let track = library.save(&Track::default()).unwrap();
        ArtistRef::attach(&library, &artist, &track.id);
        assert!(track.artists(&library).len() == 1);
    }
}

