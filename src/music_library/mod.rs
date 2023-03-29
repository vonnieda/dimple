use std::{fmt::Debug};

use image::DynamicImage;
use rodio::{Sink};
use serde::{Serialize, Deserialize};

pub mod local;
pub mod image_cache;
pub mod navidrome;

// TODO Absolutely hate that the objects can't carry the library, or get back
// to it. Seems so broken and stupid. Gotta figure out it. And don't forget that
// I planned for local library to use everyone else's urls so that the library
// finder thing can find the right library.
// Maybe as long as I can find a library for the URL it'll be okay. 
pub trait Library {
    fn releases(&self) -> Result<Vec<Release>, String>;

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        todo!();
    }

    // TODO I wanted to have this return a Source but I couldn't figure out how.
    fn stream(&self, track: &Track, sink: &Sink) -> Result<(), String>{
        todo!();
    }

    fn merge_release(&self, _library: &dyn Library, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Release {
    pub url: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub art: Vec<Image>,
    pub genres: Vec<Genre>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Track {
    pub url: String,
    pub title: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Genre {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
}

