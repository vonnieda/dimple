use dimple_db::db::{Entity, transaction::DbTransaction};
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
    pub fn attach(txn: &DbTransaction, artist: &Artist, model_id: &Option<String>) -> Result<(), anyhow::Error> {
        let sql = "SELECT * FROM ArtistRef WHERE artist_id = ? and model_id = ?";
        if txn.query::<ArtistRef, _>(sql, (artist.id.as_ref(), model_id))?.is_empty() {
            let _ = txn.save(&ArtistRef {
                model_id: model_id.clone().unwrap(),
                artist_id: artist.id.clone().unwrap(),
                ..Default::default()
            })?;
        }
        Ok(())
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
        let _ = library.db.transaction(|t| {
            ArtistRef::attach(t, &artist, &track.id)?;
            ArtistRef::attach(t, &artist, &track.id)
        });
        assert_eq!(track.artists(&library).len(), 1);
    }
}

