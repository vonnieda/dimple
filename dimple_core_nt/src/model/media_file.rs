use rusqlite::Row;

use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaFile {
    pub key: Option<String>,

    pub file_path: String,

    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
}

impl FromRow for MediaFile {
    fn from_row(row: &Row) -> Self {
        Self {
            key: row.get("key").unwrap(),
            artist: row.get("artist").unwrap(),
            album: row.get("album").unwrap(),
            title: row.get("title").unwrap(),
            file_path: row.get("file_path").unwrap(),
        }
    }
}

impl Diff for MediaFile {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        // TODO incomplete, just for ref.
        let mut diff = vec![];
        if self.file_path != other.file_path {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("file_path".to_string()), 
                value: Some(other.file_path.clone()), ..Default::default() });
        }
        if self.artist != other.artist {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("artist".to_string()), 
                value: other.artist.clone(), ..Default::default() });
        }
        if self.album != other.album {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("album".to_string()), 
                value: other.album.clone(), ..Default::default() });
        }
        if self.title != other.title {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("title".to_string()), 
                value: other.title.clone(), ..Default::default() });
        }
        diff
    }
    
    fn apply_diff(&mut self, diff: &[ChangeLog]) {
        for change in diff {
            if change.op == "set" {
                if let Some(field) = change.field.clone() {
                    if &field == "file_path" {
                        self.file_path = change.value.clone().unwrap();
                    }
                    if &field == "artist" {
                        self.artist = change.value.clone();
                    }
                    if &field == "album" {
                        self.album = change.value.clone();
                    }
                    if &field == "title" {
                        self.title = change.value.clone();
                    }
                }
            }
        }
    }    
}

impl Model for MediaFile {
    fn table_name() -> String {
        "MediaFile".to_string()
    }

    fn key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn upsert(&self, conn: &rusqlite::Connection) {
        conn.execute("INSERT OR REPLACE INTO MediaFile 
            (key, artist, album, title, file_path) 
            VALUES (?1, ?2, ?3, ?4, ?6)",
            (&self.key, &self.artist, &self.album, &self.title, &self.file_path)).unwrap();
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone()
    }
}