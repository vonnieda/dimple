use std::{fmt::Debug};

use image::DynamicImage;
use serde::{Serialize, Deserialize};

pub mod local;
pub mod image_cache;
pub mod navidrome;
// pub mod directory_library;

// TODO Looking good but still struggling with how to handle the stuff that
// might need to be downloaded lazy. Not having access to the library from
// within the object is a pain. One thought is that all structs are fully
// populated, including images, and the only thing really missing is the
// stream which you have to call back into the library for. But that basically
// means hydrating the entire library and now we have ORM.
// 

pub trait Library {
    fn releases(&self) -> Result<Vec<Release>, String>;

    fn merge_release(&self, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    
    #[serde(skip)] 
    pub original: DynamicImage,
}

