use crate::library::Library;

use super::{Dimage, LibraryModel};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DimageRef {
    pub model_key: String,
    pub dimage_key: String,
}

impl DimageRef {
    pub fn attach(library: &Library, dimage: &Dimage, model: &impl LibraryModel) {
        let _ = library.conn().execute(
            "INSERT INTO DimageRef (dimage_key, model_key) VALUES (?, ?)", 
            (dimage.key.clone(), model.key()));
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Dimage, DimageRef, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let dimage = library.save(&Dimage::default());
        let track = library.save(&Track::default());
        DimageRef::attach(&library, &dimage, &track);
        assert!(track.images(&library).len() == 1);
    }
}

