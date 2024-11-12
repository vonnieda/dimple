use rusqlite::Row;

use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrackSource {
    pub key: Option<String>,
    pub track_key: String,
    pub media_file_key: Option<String>,
}

impl FromRow for TrackSource {
    fn from_row(row: &Row) -> Self {
        Self {
            key: row.get("key").unwrap(),
            track_key: row.get("track_key").unwrap(),
            media_file_key: row.get("media_file_key").unwrap(),
        }
    }
}

impl Diff for TrackSource {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        let mut diff = vec![];
        if self.key != other.key {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("key".to_string()), 
                value: other.key.clone(), ..Default::default() });
        }
        if self.track_key != other.track_key {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("track_key".to_string()), 
                value: Some(other.track_key.clone()), ..Default::default() });
        }
        if self.media_file_key != other.media_file_key {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("media_file_key".to_string()), 
                value: other.media_file_key.clone(), ..Default::default() });
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
                    if &field == "track_key" {
                        self.track_key = change.value.clone().unwrap();
                    }
                    if &field == "media_file_key" {
                        self.media_file_key = change.value.clone();
                    }
                }
            }
        }
    }    
}

impl Model for TrackSource {
    fn table_name() -> String {
        "TrackSource".to_string()
    }

    fn key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone()
    }

    fn upsert(&self, conn: &rusqlite::Connection) {
        conn.execute("INSERT OR REPLACE INTO TrackSource 
            (key, track_key, media_file_key) 
            VALUES (?1, ?2, ?3)",
            (&self.key, &self.track_key, &self.media_file_key)).unwrap();
    }

    fn log_changes() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, TrackSource}};

    #[test]
    fn library_crud() {
        let library = Library::open(":memory:");
        let mut model = library.save(&TrackSource::default());
        assert!(model.key.is_some());
        assert!(model.media_file_key.is_none());
        model.media_file_key = Some("media_file_key".to_string());
        let model = library.save(&model);
        let model: TrackSource = library.get(&model.key.unwrap()).unwrap();
        assert!(model.media_file_key == Some("media_file_key".to_string()));
    }

    #[test]
    fn diff() {
        let a = TrackSource::default();
        let b = TrackSource {
            key: Some("key".to_string()),
            track_key: "track_key".to_string(),
            media_file_key: Some("media_file_key".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 3);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("track_key".to_string()));
        assert!(diff[2].field == Some("media_file_key".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = TrackSource::default();
        let b = TrackSource {
            key: Some("key".to_string()),
            track_key: "track_key".to_string(),
            media_file_key: Some("media_file_key".to_string()),
        };
        let diff = a.diff(&b);
        let mut c = TrackSource::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}