
use rusqlite::Row;
use sha2::{Sha256, Digest};
use symphonia::core::checksum::Md5;
use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq)]
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

impl FromRow for Blob {
    fn from_row(row: &Row) -> Self {
        Self {
            key: row.get("key").unwrap(),
            sha256: row.get("sha256").unwrap(),
            length: row.get("length").unwrap(),
        }
    }
}

impl Diff for Blob {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        let mut diff = vec![];
        if self.key != other.key {
            diff.push(ChangeLog { model: "Blob".to_string(), 
                op: "set".to_string(), field: Some("key".to_string()), 
                value: other.key.clone(), ..Default::default() });
        }
        if self.sha256 != other.sha256 {
            diff.push(ChangeLog { model: "Blob".to_string(), 
                op: "set".to_string(), field: Some("sha256".to_string()), 
                value: Some(other.sha256.clone()), ..Default::default() });
        }
        if self.length != other.length {
            diff.push(ChangeLog { model: "Blob".to_string(), 
                op: "set".to_string(), field: Some("length".to_string()), 
                value: Some(other.length.to_string()), ..Default::default() });
        }
        diff
    }
    
    fn apply_diff(&mut self, diff: &[ChangeLog]) {
        for change in diff {
            if change.op == "set" {
                if let Some(field) = change.field.clone() {
                    if &field == "key" {
                        self.key = change.value.clone();
                    }
                    if &field == "length" {
                        let src = change.value.clone().unwrap();
                        self.length = u64::from_str_radix(&src, 10).unwrap();
                    }
                    if &field == "sha256" {
                        self.sha256 = change.value.clone().unwrap();
                    }
                }
            }
        }
    }    
}

impl Model for Blob {
    fn table_name() -> String {
        "Blob".to_string()
    }

    fn key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn upsert(&self, conn: &rusqlite::Connection) {
        conn.execute("INSERT OR REPLACE INTO Blob 
            (key, sha256, length) 
            VALUES (?1, ?2, ?3)",
            (&self.key, &self.sha256, &self.length)).unwrap();
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone()
    }
        
    fn log_changes() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::Diff};

    use super::Blob;

    #[test]
    fn library_crud() {
        let library = Library::open("file:6384d9e0-74c1-4ecd-9ea3-b5d0198f134e?mode=memory&cache=shared");
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

    #[test]
    fn read() {
        let a = Blob::read("tests/artifacts/hello.txt");
        assert!(&a.sha256 == "319b0878313c131df1382eaac03be8ef59d466f81d16717c751368da578051ca");
        assert!(a.length == 28);
    }
}