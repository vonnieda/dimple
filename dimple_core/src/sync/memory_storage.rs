use std::{collections::HashMap, sync::{Arc, RwLock}};

use super::storage::Storage;

#[derive(Default, Clone)]
pub struct MemoryStorage {
    map: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl Storage for MemoryStorage {
    fn put_object(&self, path: &str, contents: &[u8]) {
        self.map.write().unwrap().insert(path.to_owned(), contents.to_vec());
    }

    fn get_object(&self, path: &str) -> Option<Vec<u8>> {
        let obj = self.map.read().unwrap().get(path).cloned();
        obj
    }

    fn list_objects(&self, storage_prefix: &str) -> Vec<String> {
        self.map.read().unwrap().keys()
            .filter(|key| key.starts_with(storage_prefix))
            .cloned().collect()
    }
}

