/// Objects all have a URL. The URL must uniquely identify that object and
/// it should, at least, have a scheme that matches the library it
/// came from. In other words, given the same library config, it should be
/// possible to re-load the object from the library with the same URL.

use std::{fmt::Debug};

use image::DynamicImage;
use rodio::{Sink};
use serde::{Serialize, Deserialize};

pub mod local;
pub mod image_cache;
pub mod navidrome;
pub mod libraries;

pub trait Library: Send + Sync {
    fn releases(&self) -> Result<Vec<Release>, String>;

    fn image(&self, _image: &Image) -> Result<DynamicImage, String>;

    // TODO I wanted to have this return a Source but I couldn't figure out how.
    fn stream(&self, _track: &Track, _sink: &Sink) -> Result<(), String>;

    fn merge_release(&self, _library: &dyn Library, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

