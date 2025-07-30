use dimple_db::db::Entity;
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
    pub fn attach(library: &Library, link: &Link, model_id: &Option<String>) {
        let _ = library.save(&LinkRef {
            model_id: model_id.clone().unwrap(),
            link_id: link.id.clone().unwrap(),
            ..Default::default()
        });
    }    
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Link, LinkRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let link = library.save(&Link::default());
        let track = library.save(&Track::default());
        LinkRef::attach(&library, &link, &track.id);
        assert!(track.links(&library).len() == 1);
    }
}

