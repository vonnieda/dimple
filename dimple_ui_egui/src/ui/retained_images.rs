use std::{sync::{Arc, RwLock, Mutex}, collections::HashMap};

use dimple_core::{library::LibraryHandle, model::Image};
use eframe::epaint::{Color32, ColorImage};
use egui_extras::RetainedImage;
use threadpool::ThreadPool;

use super::utils;

pub struct RetainedImages {
    // TODO see if we can cache this on disk
    // for fast startup.
    retained_images: Arc<RwLock<HashMap<String, Arc<RetainedImage>>>>,
    thread_pool: Arc<Mutex<ThreadPool>>,
    library: LibraryHandle,
}

impl RetainedImages {
    pub fn new(library: LibraryHandle) -> Self {
        // std::thread::spawn(|| {
        //     loop {
        //         log::info!("{} {}", self.retained_im)
        //         std::thread::sleep(Duration::from_secs_f32(5.0));
        //     }
        // });
        Self {
            retained_images: Arc::new(RwLock::new(HashMap::new())),
            thread_pool: Arc::new(Mutex::new(ThreadPool::default())),
            library: library.clone(),
        }
    }

    /// Get a thumbnail for the given Image, returning a RetainedImage.
    /// Caches for performance. Unbounded for now.
    /// Requests the image from the Librarian if it's not in the cache.
    pub fn get(&self, image: &Image, 
        width: usize, height: usize) -> Arc<RwLock<Arc<RetainedImage>>> {
        
        let key = format!("{}:{}x{}", image.url, width, height);
        
        if let Some(image) = self.retained_images.read().unwrap().get(&key) {
            // TODO gotta be a better way than creating a new lock every time. 
            // Probably needs to be stored as the lock.
            return Arc::new(RwLock::new(image.clone()));
        }

        // TODO needs variable name cleanup and maybe turn some of this into
        // a function, or even a class
        let placeholder = ColorImage::new([width, height], Color32::BLACK);
        let retained_arc = Arc::new(RetainedImage::from_color_image("", placeholder));
        self.retained_images.write().unwrap().insert(key.clone(), retained_arc.clone());
        let retained = Arc::new(RwLock::new(retained_arc));

        let library_1 = self.library.clone();
        let image_1 = image.clone();
        let retained_images_1 = self.retained_images.clone();
        let retained_1 = retained.clone();
        self.thread_pool.lock().unwrap().execute(move || {
            if let Ok(dynamic) = library_1.image(&image_1) {
                let new_retained = Arc::new(utils::dynamic_to_retained("", &dynamic));
                retained_images_1.write().unwrap().insert(key, new_retained.clone());
                *retained_1.write().unwrap() = new_retained;
            }
        });

        retained
    }
}
