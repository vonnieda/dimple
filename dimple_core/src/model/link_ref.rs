use dimple_db::db::{Entity, transaction::DbTransaction};
use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Link};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LinkRef {
    pub id: Option<String>,
    pub model_id: String,
    pub link_id: String,
}

impl LinkRef {
    pub fn attach(txn: &DbTransaction, link: &Link, model_id: &Option<String>) -> Result<(), anyhow::Error> {
        let sql = "SELECT * FROM LinkRef WHERE link_id = ? and model_id = ?";
        if txn.query::<LinkRef, _>(sql, (link.id.as_ref(), model_id))?.is_empty() {
            let _ = txn.save(&LinkRef {
                model_id: model_id.clone().unwrap(),
                link_id: link.id.clone().unwrap(),
                ..Default::default()
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Link, LinkRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let link = library.save(&Link::default()).unwrap();
        let track = library.save(&Track::default()).unwrap();
        let _ = library.db.transaction(|t| LinkRef::attach(t, &link, &track.id));
        assert!(track.links(&library).len() == 1);
    }
}

