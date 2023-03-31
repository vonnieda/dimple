use std::{fmt::Debug, sync::{Arc, RwLock}, mem, collections::HashMap, time::Duration};

use crossbeam::channel::{unbounded, Receiver};

use super::{Library, Release, Image, Track};


#[derive(Clone)]
pub struct Libraries {
    libraries: Arc<RwLock<Vec<Arc<Box<dyn Library>>>>>,

    // TODO this will be used for merging results before sending them
    // downstream.
    releases_by_url: Arc<RwLock<HashMap<String, Release>>>,
}

impl Libraries {    
    pub fn new() -> Self {
        let libraries: Arc<RwLock<Vec<Arc<Box<dyn Library>>>>> = Default::default();
        let releases_by_url = Arc::new(RwLock::new(HashMap::new()));

        Self {
            libraries,
            releases_by_url
        }
    }

    pub fn add_library(&mut self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(Arc::new(library));
    }
}

impl Library for Libraries {
    fn name(&self) -> String {
        // TODO the list
        return "Libraries".to_string();
    }

    fn releases(&self) -> Receiver<Release> {
        let (sender, receiver) = unbounded::<Release>();
        for library in self.libraries.read().unwrap().iter() {
            let sender = sender.clone();
            let library = library.clone();
            std::thread::spawn(move || {
                for release in library.releases() {
                    sender.send(release).unwrap();
                }
            });
        }
        return receiver;
    }

    fn image(&self, image: &Image) -> Result<image::DynamicImage, String> {
        for library in self.libraries.read().unwrap().iter() {
            if let Ok(image) = library.image(image) {
                return Ok(image);
            }
        }
        Err("Not found".to_string())
    }

    fn stream(&self, track: &Track, sink: &rodio::Sink) -> Result<(), String>{
        for library in self.libraries.read().unwrap().iter() {
            if library.stream(track, sink).is_ok() {
                return Ok(());
            }
        }
        Err("Not found".to_string())
    }

    fn merge_release(&self, _library: &dyn Library, _release: &super::Release) -> Result<(), String> {
        Err("moof".to_string())
    }
}

impl Debug for dyn Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

