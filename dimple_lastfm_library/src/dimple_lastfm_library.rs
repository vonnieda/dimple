use dimple_core::library::{Library, DimpleEntity};

#[derive(Debug, Default)]
pub struct LastFmLibrary {
}

impl LastFmLibrary {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Library for LastFmLibrary {
    fn name(&self) -> String {
        "last.fm".to_string()
    }

    // fn search(&self, _query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
    //     Box::new(vec![].into_iter())
    // }

    // fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::DimpleArtist>> {
    //     Box::new(vec![].into_iter())
    // }

    // // reqwest::blocking::get("http://ws.audioscrobbler.com/2.0/?method=artist.getinfo&artist=Cher&api_key=YOUR_API_KEY&format=json")
    // fn image(&self, _entity: &LibraryEntity) -> Option<image::DynamicImage> {
    //     None
    // }
}