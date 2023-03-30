use std::{fmt::Debug, sync::Arc, mem};

use crossbeam::channel::{unbounded, Receiver};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::{Library, Release, Image, Track};


/// Libraries implements a manager of sorts for multiple libraries. It monitors
/// each library in its list for changes and integrates those changes into
/// its central list, which is used to respond to data requests. Requests
/// for images and streams are automatically delegated to the appropriate
/// library based on URL.
#[derive(Clone, Default)]
pub struct Libraries {
    libraries: Vec<Arc<Box<dyn Library>>>,
}

impl Libraries {
    pub fn library_for_url(&self, url: &str) -> Option<Arc<Box<dyn Library>>> {
        None
    }

    pub fn add_library(&mut self, library: Box<dyn Library>) {
        self.libraries.push(Arc::new(library));
    }
}

impl Debug for dyn Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Library for Libraries {
    fn releases(&self) -> Result<Vec<Release>, String> {
        let mut releases: Vec<Release> = Vec::new();
        for library in &self.libraries {
            releases.extend(library.releases().unwrap_or(vec![]));
        }
        log::info!("{} releases, taking {} bytes", 
            &releases.len(), 
            mem::size_of_val(&releases));
        return Ok(releases);
    }

    fn releases_stream(&self) -> Receiver<Release> {
        // Launch a thread for each library that will feed
        // us releases.
        // TODO actually, I can see this essentially being the sync
        // function once I have merging.
        let (sender, receiver) = unbounded::<Release>();
        for library in &self.libraries {
            let library = library.clone();
            let sender = sender.clone();
            std::thread::spawn(move || {
                println!("grabbing releases from {:?}", library);
                for release in library.releases_stream() {
                    sender.send(release).unwrap();
                }
            });
        }
        return receiver;
    }

    fn image(&self, image: &Image) -> Result<image::DynamicImage, String> {
        for library in &self.libraries {
            if let Ok(image) = library.image(image) {
                return Ok(image);
            }
        }
        Err("Not found".to_string())
    }

    fn stream(&self, track: &Track, sink: &rodio::Sink) -> Result<(), String>{
        for library in &self.libraries {
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