
use dimple_core_macro::ModelSupport;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Blob {
    pub key: Option<String>,
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
            key: None,
            sha256: sha256,
            length: content.len() as u64,
        }
    }    

    fn calculate_sha256(data: &Vec<u8>) -> String {
        // blake3::hash(data).to_string()

        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }        
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::Diff};

    use super::Blob;

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Blob::default());
        assert!(model.key.is_some());
        model.sha256 = "sha256".to_string();
        let model = library.save(&model);
        let model: Blob = library.get(&model.key.unwrap()).unwrap();
        assert!(model.sha256 == "sha256".to_string());
    }

    #[test]
    fn diff() {
        let a = Blob::default();
        let b = Blob {
            key: Some("key".to_string()),
            sha256: "sha256".to_string(),
            length: 100,
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 3);
        assert!(diff[0].model == "Blob".to_string());
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("sha256".to_string()));
        assert!(diff[2].field == Some("length".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Blob::default();
        let b = Blob {
            key: Some("key".to_string()),
            sha256: "sha256".to_string(),
            length: 100,
        };
        let diff = a.diff(&b);
        let mut c = Blob::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }

    // TODO temp commented out cause windows.
    // #[test]
    // fn read() {
    //     let a = Blob::read("tests/data/hello.txt");
    //     assert!(&a.sha256 == "319b0878313c131df1382eaac03be8ef59d466f81d16717c751368da578051ca");
    //     assert!(a.length == 28);
    // }
}