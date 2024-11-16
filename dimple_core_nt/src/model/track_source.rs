use dimple_core_nt_macro::ModelSupport;
use rusqlite::Row;

use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct TrackSource {
    pub key: Option<String>,
    pub track_key: String,
    pub blob_key: Option<String>,
}

// impl FromRow for TrackSource {
//     fn from_row(row: &Row) -> Self {
//         Self {
//             key: row.get("key").unwrap(),
//             track_key: row.get("track_key").unwrap(),
//             blob_key: row.get("blob_key").unwrap(),
//         }
//     }
// }

// impl Diff for TrackSource {
//     fn diff(&self, other: &Self) -> Vec<ChangeLog> {
//         let mut diff = vec![];
//         if self.key != other.key {
//             diff.push(ChangeLog { model: "TrackSource".to_string(), 
//                 op: "set".to_string(), field: Some("key".to_string()), 
//                 value: other.key.clone(), ..Default::default() });
//         }
//         if self.track_key != other.track_key {
//             diff.push(ChangeLog { model: "TrackSource".to_string(), 
//                 op: "set".to_string(), field: Some("track_key".to_string()), 
//                 value: Some(other.track_key.clone()), ..Default::default() });
//         }
//         if self.blob_key != other.blob_key {
//             diff.push(ChangeLog { model: "TrackSource".to_string(), 
//                 op: "set".to_string(), field: Some("blob_key".to_string()), 
//                 value: other.blob_key.clone(), ..Default::default() });
//         }
//         diff
//     }
    
//     fn apply_diff(&mut self, diff: &[ChangeLog]) {
//         for change in diff {
//             if change.op == "set" {
//                 if let Some(field) = change.field.clone() {
//                     if &field == "key" {
//                         self.key = change.value.clone();
//                     }
//                     if &field == "track_key" {
//                         self.track_key = change.value.clone().unwrap();
//                     }
//                     if &field == "blob_key" {
//                         self.blob_key = change.value.clone();
//                     }
//                 }
//             }
//         }
//     }    
// }

// impl Model for TrackSource {
//     fn table_name() -> String {
//         "TrackSource".to_string()
//     }

//     fn key(&self) -> Option<String> {
//         self.key.clone()
//     }
    
//     fn set_key(&mut self, key: Option<String>) {
//         self.key = key.clone()
//     }

//     fn upsert(&self, conn: &rusqlite::Connection) {
//         conn.execute("INSERT OR REPLACE INTO TrackSource 
//             (key, track_key, blob_key) 
//             VALUES (?1, ?2, ?3)",
//             (&self.key, &self.track_key, &self.blob_key)).unwrap();
//     }

//     fn log_changes() -> bool {
//         true
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, TrackSource}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:712f9444-5755-4795-a75f-a4c33fd695c6?mode=memory&cache=shared");
        let mut model = library.save(&TrackSource::default());
        assert!(model.key.is_some());
        assert!(model.blob_key.is_none());
        model.blob_key = Some("blob_key".to_string());
        let model = library.save(&model);
        let model: TrackSource = library.get(&model.key.unwrap()).unwrap();
        assert!(model.blob_key == Some("blob_key".to_string()));
    }

    #[test]
    fn diff() {
        let a = TrackSource::default();
        let b = TrackSource {
            key: Some("key".to_string()),
            track_key: "track_key".to_string(),
            blob_key: Some("blob_key".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 3);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("track_key".to_string()));
        assert!(diff[2].field == Some("blob_key".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = TrackSource::default();
        let b = TrackSource {
            key: Some("key".to_string()),
            track_key: "track_key".to_string(),
            blob_key: Some("blob_key".to_string()),
        };
        let diff = a.diff(&b);
        let mut c = TrackSource::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}