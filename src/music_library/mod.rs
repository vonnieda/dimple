use std::{fmt::Debug};

use image::DynamicImage;

pub mod local;
pub mod image_cache;
pub mod navidrome;
// pub mod directory_library;

pub trait Library {
    fn releases(&self) -> Result<Vec<Release>, String>;
    
    fn merge_release(&self, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Release {
    pub url: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub art: Vec<Image>,
    pub genres: Vec<Genre>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Artist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Track {
    pub url: String,
    pub title: String,
    pub art: Vec<Image>,
}


#[derive(Default, Clone, Debug, PartialEq)]
pub struct Genre {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Image {
    pub url: String,
    pub original: DynamicImage,
}

