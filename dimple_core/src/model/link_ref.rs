use crate::library::Library;

use super::{LibraryModel, Link};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LinkRef {
    pub model_key: String,
    pub link_key: String,
}

impl LinkRef {
    pub fn attach(library: &Library, link: &Link, model: &impl LibraryModel) {
        let _ = library.conn().execute(
            "INSERT INTO LinkRef (link_key, model_key) VALUES (?, ?)", 
            (link.key.clone(), model.key()));
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
        LinkRef::attach(&library, &link, &track);
        assert!(track.links(&library).len() == 1);
    }
}

