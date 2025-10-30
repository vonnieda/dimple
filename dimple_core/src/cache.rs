use std::path::PathBuf;

pub struct Cache {

}

impl Cache {
    pub fn open_memory() -> anyhow::Result<Self> {
        todo!()
    }

    pub fn open_path(path: &PathBuf) -> anyhow::Result<Self> {
        todo!()
    }

    // pub fn cache_get(&self, url: &str) -> Option<CachedResponse> {
    //     let bytes = cacache::read_sync(self.cache_dir.clone(), url).ok()?;
    //     serde_json::from_slice(&bytes).ok()
    // }

    // pub fn cache_put(&self, url: &str, response: &CachedResponse) {
    //     let bytes = serde_json::to_vec(response).unwrap();
    //     cacache::write_sync(self.cache_dir.clone(), url, &bytes).unwrap();
    // }
}