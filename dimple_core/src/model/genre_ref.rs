use dimple_db::db::Entity;
use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Genre};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GenreRef {
    pub id: Option<String>,
    pub model_id: String,
    pub genre_id: String,
}

impl GenreRef {
    pub fn attach(library: &Library, genre: &Genre, model_id: &Option<String>) {
        library.db.transaction(|txn| {
            let sql = "SELECT * FROM GenreRef WHERE genre_id = ? and model_id = ?";
            if txn.query::<GenreRef, _>(sql, (genre.id.as_ref(), model_id))?.is_empty() {
                let _ = txn.save(&GenreRef {
                    model_id: model_id.clone().unwrap(),
                    genre_id: genre.id.clone().unwrap(),
                    ..Default::default()
                })?;
            }
            Ok(())
        }).unwrap();
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
        }).unwrap();
        let track = library.save(&Track::default()).unwrap();
        GenreRef::attach(&library, &genre, &track.id);
        assert!(track.genres(&library).len() == 1);
    }
}

