use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blob {
    pub key: Option<String>,
    pub sha256: String,
    pub length: u64,
}

impl Blob {
    pub fn from_path(path: &str) -> Self {
        let path = std::fs::canonicalize(path).unwrap();
        let content = std::fs::read(&path).unwrap();
        let sha256 = Self::calculate_sha256(&content);
        Self {
            key: None,
            sha256: sha256,
            length: content.len() as u64,
        }
    }    

    fn calculate_sha256(data: &Vec<u8>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }        
}
