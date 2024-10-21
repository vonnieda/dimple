pub trait Storage {
    fn put_object(&self, path: &str, contents: &[u8]);
    fn get_object(&self, path: &str) -> Option<Vec<u8>>;
    fn list_objects(&self, storage_prefix: &str) -> Vec<String>;
}

