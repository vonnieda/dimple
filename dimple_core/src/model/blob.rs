
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Blob {
    pub id: Option<String>,
    // echo "Hello and Welcome to Dimple" | sha256sum 
    // 319b0878313c131df1382eaac03be8ef59d466f81d16717c751368da578051ca  -
    // echo "Hello and Welcome to Dimple" | b3sum
    // 8908ecf28db1d115047a8917f22f5bd0bf8b7b49fee2f73fb17b324e5ad60b1a  -    
    // TODO check blake3, claude says up to 10x faster
    // https://github.com/BLAKE3-team/BLAKE3
    // Did a quick test and it didn't seem faster, but try more.
    pub sha256: String,
    pub length: u64,
}

impl Blob {
    pub fn read(path: &str) -> Self {
        let path = std::fs::canonicalize(path).unwrap();
        let content = std::fs::read(&path).unwrap();
        let sha256 = Self::calculate_sha256(&content);
        Self {
            id: None,
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

#[cfg(test)]
mod tests {
    use crate::{library::Library};

    use super::Blob;

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Blob::default());
        assert!(model.id.is_some());
        model.sha256 = "sha256".to_string();
        let model = library.save(&model);
        let model: Blob = library.get(&model.id.unwrap()).unwrap();
        assert!(model.sha256 == "sha256".to_string());
    }

    // TODO temp commented out cause windows.
    // #[test]
    // fn read() {
    //     let a = Blob::read("tests/data/hello.txt");
    //     assert!(&a.sha256 == "319b0878313c131df1382eaac03be8ef59d466f81d16717c751368da578051ca");
    //     assert!(a.length == 28);
    // }
}