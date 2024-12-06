use anyhow::Error;
use dimple_core::model::{Artist, Entity, Genre, Medium, Model, Release, ReleaseGroup};

use crate::librarian::Librarian;

pub trait Hydrate {
    fn hydrate(&self, librarian: &Librarian) -> Result<Self, Error>
    where
        Self: Sized;
}

impl Hydrate for Model {
    fn hydrate(&self, librarian: &Librarian) -> Result<Self, Error>
    where
        Self: Sized {

        match self {
            Model::Artist(artist) => Ok(artist.hydrate(librarian)?.model()),
            _ => todo!()
        }
    }
}

impl Hydrate for Artist {
    fn hydrate(&self, librarian: &Librarian) -> Result<Self, Error> {
        let mut result = librarian.get2(self.clone())?;
        result.genres = librarian
            .list2(Genre::default(), Some(result.clone()))?
            .collect();
        Ok(result)
    }
}

impl Hydrate for ReleaseGroup {
    fn hydrate(&self, librarian: &Librarian) -> Result<Self, Error> {
        let mut result = librarian.get2(self.clone())?;
        result.genres = librarian
            .list2(Genre::default(), Some(result.clone()))?
            .collect();
        // TODO artist credits
        Ok(result)
    }
}

impl Hydrate for Release {
    fn hydrate(&self, librarian: &Librarian) -> Result<Self, Error> {
        let mut result = librarian.get2(self.clone())?;
        result.genres = librarian
            .list2(Genre::default(), Some(result.clone()))?
            .collect();
        result.media = librarian
            .list2(Medium::default(), Some(result.clone()))?
            .collect();
        // TODO artist credits
        Ok(result)
    }
}
