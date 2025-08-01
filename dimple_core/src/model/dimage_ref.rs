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
        let _ = library.save(&DimageRef {
            model_id: model_id.clone().unwrap(),
            dimage_id: dimage.id.clone().unwrap(),
            ..Default::default()
        });
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

