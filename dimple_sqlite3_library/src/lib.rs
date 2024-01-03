use std::{path::Path, fs::File, collections::{hash_map, HashMap, HashSet}, time::Duration, io::Error, sync::{mpsc::{channel, Receiver}, RwLock}};

use audiotags::{Tag, AudioTag, AudioTagEdit, AudioTagConfig};
use dimple_core::{library::Library, model::{Release, Track, Artist, Genre}};
use image::DynamicImage;
use walkdir::{WalkDir, DirEntry};
use dimple_core::model::Image;

pub struct Sqlite3Library {
    path: String,
    images_by_url: RwLock<HashMap<String, DynamicImage>>,
}

impl Sqlite3Library {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            images_by_url: RwLock::new(HashMap::new()),
        }
    }
}

impl Library for Sqlite3Library {
    fn name(&self) -> String {
        format!("Sqlite3Library({})", self.path)
    }
}
