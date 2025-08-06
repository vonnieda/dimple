use dimple_db::db::Entity;
use serde::{Deserialize, Serialize};

use crate::library::{Library};

use super::{Dimage};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DimageRef {
    pub id: Option<String>,
    pub model_id: String,
    pub dimage_id: String,
}

impl DimageRef {
    pub fn attach(library: &Library, dimage: &Dimage, model_id: &Option<String>) {
        library.db.transaction(|txn| {
            let sql = "SELECT * FROM DimageRef WHERE dimage_id = ? and model_id = ?";
            if txn.query::<DimageRef, _>(sql, (dimage.id.as_ref(), model_id))?.is_empty() {
                let _ = txn.save(&DimageRef {
                    model_id: model_id.clone().unwrap(),
                    dimage_id: dimage.id.clone().unwrap(),
                    ..Default::default()
                })?;
            }
            Ok(())
        }).unwrap();
    }    
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{library::Library, model::{Dimage, DimageRef, Track}};

    #[test]
    fn library_crud() -> Result<()> {
        let library = Library::open_memory();
        let dimage = library.save(&Dimage::default())?;
        let track = library.save(&Track::default())?;
        DimageRef::attach(&library, &dimage, &track.id);
        assert!(track.images(&library).len() == 1);
        Ok(())
    }
}

