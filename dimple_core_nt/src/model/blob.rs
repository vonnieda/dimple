use std::{path::{Path, PathBuf}, time::Instant};

use rusqlite::Row;
use sha2::{Sha256, Digest};
use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Blob {
    pub key: Option<String>,
    // echo "Hello and Welcome to Dimple" | sha256sum 
    // 319b0878313c131df1382eaac03be8ef59d466f81d16717c751368da578051ca  -
    pub sha256: String,
    pub length: u64,
    pub local_path: Option<String>,
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
            local_path: Some(path.to_str().unwrap().to_owned()),
        }
    }    

    fn calculate_sha256(data: &Vec<u8>) -> String {
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
            local_path: row.get("local_path").unwrap(),
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
        if self.local_path != other.local_path {
            diff.push(ChangeLog { model: "Blob".to_string(), 
                op: "set".to_string(), field: Some("local_path".to_string()), 
                value: other.local_path.clone(), ..Default::default() });
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
                    if &field == "local_path" {
                        self.local_path = change.value.clone();
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
            (key, sha256, length, local_path) 
            VALUES (?1, ?2, ?3, ?4)",
            (&self.key, &self.sha256, &self.length, &self.local_path)).unwrap();
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
        let library = Library::open(":memory:");
        let mut model = library.save(&Blob::default());
        assert!(model.key.is_some());
        assert!(model.local_path.is_none());
        model.local_path = Some("local_path".to_string());
        let model = library.save(&model);
        let model: Blob = library.get(&model.key.unwrap()).unwrap();
        assert!(model.local_path == Some("local_path".to_string()));
    }

    #[test]
    fn diff() {
        let a = Blob::default();
        let b = Blob {
            key: Some("key".to_string()),
            sha256: "file_path".to_string(),
            length: 100,
            local_path: Some("local_path".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 4);
        assert!(diff[0].model == "Blob".to_string());
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("sha256".to_string()));
        assert!(diff[2].field == Some("length".to_string()));
        assert!(diff[3].field == Some("local_path".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Blob::default();
        let b = Blob {
            key: Some("key".to_string()),
            sha256: "file_path".to_string(),
            length: 100,
            local_path: Some("local_path".to_string()),
        };
        let diff = a.diff(&b);
        let mut c = Blob::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }

    #[test]
    fn read() {
        let a = Blob::read("tests/hello.txt");
        assert!(&a.sha256 == "319b0878313c131df1382eaac03be8ef59d466f81d16717c751368da578051ca");
        assert!(a.length == 28);
    }
}