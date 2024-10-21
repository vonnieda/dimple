use std::{collections::HashMap, sync::RwLock};

use super::storage::Storage;

#[derive(Default)]
pub struct MemoryStorage {
    map: RwLock<HashMap<String, Vec<u8>>>,
}

impl Storage for MemoryStorage {
    fn put_object(&self, path: &str, contents: &[u8]) {
        self.map.write().unwrap().insert(path.to_owned(), contents.to_vec());
    }

    fn get_object(&self, path: &str) -> Option<Vec<u8>> {
        self.map.read().unwrap().get(path).cloned()
    }

    fn list_objects(&self, storage_prefix: &str) -> Vec<String> {
        self.map.read().unwrap().keys()
            .filter(|key| key.starts_with(storage_prefix))
            .cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::storage::Storage;

    use super::MemoryStorage;

    #[test]
    fn basics() {
        let storage = MemoryStorage::default();
        let objects = storage.list_objects("");
        assert!(objects.len() == 0);
        storage.put_object("001/001.db", "faa44c67-92e2-411a-808b-cfd9fc9a263a".as_bytes());
        storage.put_object("001/002.db", "dcf51319-0a0d-4d1a-b825-26dea74d861b".as_bytes());
        storage.put_object("001/003.db", "1d922942-197d-4e72-9ccf-421e9f4d61aa".as_bytes());
        storage.put_object("002/002.db", "4a88dbef-349d-4845-bdab-6ed97457217d".as_bytes());
        storage.put_object("002/001.db", "fa63f446-d237-41bf-9739-30539223bf02".as_bytes());
        let objects = storage.list_objects("");
        assert!(objects.len() == 5);
        let objects = storage.list_objects("001/");
        assert!(objects.len() == 3);
        let objects = storage.list_objects("002/");
        assert!(objects.len() == 2);
        let content = storage.get_object("001/001.db");
        assert!(content == Some("faa44c67-92e2-411a-808b-cfd9fc9a263a".as_bytes().to_vec()));
    }
}