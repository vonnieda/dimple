pub trait Storage: Send + Sync {
    // TODO needs errors
    fn put_object(&self, path: &str, contents: &[u8]);
    fn get_object(&self, path: &str) -> Option<Vec<u8>>;
    fn list_objects(&self, path: &str) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use crate::sync::{memory_storage::MemoryStorage, s3_storage::S3Storage, storage::Storage};

    #[test]
    fn it_works() {
        let s3 = S3Storage::default();
        let memory = MemoryStorage::default();
        basics(&s3);
        basics(&memory);
    }

    fn basics(storage: &dyn Storage) {
        storage.put_object("001.db", "faa44c67-92e2-411a-808b-cfd9fc9a263a".as_bytes());
        storage.put_object("001.db/001.db", "faa44c67-92e2-411a-808b-cfd9fc9a263a".as_bytes());
        storage.put_object("001.db/002.db", "dcf51319-0a0d-4d1a-b825-26dea74d861b".as_bytes());
        let objects = storage.list_objects("001.db");
        assert!(objects.len() == 3);
    }
}