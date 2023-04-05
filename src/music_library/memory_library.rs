use std::{collections::HashMap, sync::RwLock, io::Cursor};

use image::DynamicImage;
use rodio::Decoder;

use super::{Library, Release, Image};

// TODO I think it makes sense to have like a limited length queue for the
// image and audio cache. And I guess functions for putting things in them.
// TODO I think I can make a generic library cache based on this and then
// decide if I want to cache each library at least a little bit. The releases
// probably at least.
#[derive(Debug, Default)]
pub struct MemoryLibrary {
    releases: RwLock<HashMap<String, Release>>,
    images: RwLock<HashMap<String, DynamicImage>>,
    streams: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryLibrary {
    pub fn merge_image(&self, image: &Image, dynamic_image: &DynamicImage) {
        self.images.write().unwrap().insert(image.url.clone(), dynamic_image.clone());
    }

    pub fn releases_len(&self) -> usize {
        self.releases.read().unwrap().len()
    }
}

impl Library for MemoryLibrary {
    fn name(&self) -> String {
        "Memory".to_string()
    }

    fn releases(&self) -> crossbeam::channel::Receiver<super::Release> {
        let (sender, receiver) = crossbeam::channel::unbounded();
        for release in self.releases.read().unwrap().values() {
            sender.send(release.clone()).unwrap();
        }
        receiver
    }

    fn image(&self, _image: &super::Image) -> Result<image::DynamicImage, String> {
        if let Some(image) = self.images.read().unwrap().get(&_image.url) {
            return Ok(image.clone());
        }
        Err("Not found".to_string())
    }

    fn stream(&self, _track: &super::Track, _sink: &rodio::Sink) -> Result<(), String> {
        if let Some(stream) = self.streams.read().unwrap().get(&_track.url) {
            let source = Decoder::new(Cursor::new(stream.clone())).unwrap();
            _sink.append(source);
            return Ok(());
        }
        Err("Not found".to_string())
    }

    fn merge_release(&self, _library: &dyn Library, _release: &Release) -> Result<(), String> {
        self.releases.write().unwrap().insert(_release.url.clone(), _release.clone());
        Ok(())
    }
}