use rusqlite::Row;

use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub liked: bool,
}

impl FromRow for Track {
    fn from_row(row: &Row) -> Self {
        Self {
            key: row.get("key").unwrap(),
            artist: row.get("artist").unwrap(),
            album: row.get("album").unwrap(),
            title: row.get("title").unwrap(),
            liked: row.get("liked").unwrap(),
        }
    }
}

impl Diff for Track {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        // TODO incomplete, just for ref.
        let mut diff = vec![];
        if self.artist != other.artist {
            diff.push(ChangeLog { model: "Track".to_string(), 
                op: "set".to_string(), field: Some("artist".to_string()), 
                value: other.artist.clone(), ..Default::default() });
        }
        if self.album != other.album {
            diff.push(ChangeLog { model: "Track".to_string(), 
                op: "set".to_string(), field: Some("album".to_string()), 
                value: other.album.clone(), ..Default::default() });
        }
        if self.title != other.title {
            diff.push(ChangeLog { model: "Track".to_string(), 
                op: "set".to_string(), field: Some("title".to_string()), 
                value: other.title.clone(), ..Default::default() });
        }
        if self.liked != other.liked {
            diff.push(ChangeLog { model: "Track".to_string(), 
                op: "set".to_string(), field: Some("liked".to_string()), 
                value: Some(other.liked.to_string()), ..Default::default() });
        }
        diff
    }
    
    fn apply_diff(&mut self, diff: &[ChangeLog]) {
        for change in diff {
            if change.op == "set" {
                if let Some(field) = change.field.clone() {
                    if &field == "artist" {
                        self.artist = change.value.clone();
                    }
                    if &field == "album" {
                        self.album = change.value.clone();
                    }
                    if &field == "title" {
                        self.title = change.value.clone();
                    }
                    if &field == "liked" {
                        self.liked = change.value.clone() == Some("true".to_string());
                    }
                }
            }
        }
    }        
}

impl Model for Track {
    fn table_name() -> String {
        "Track".to_string()
    }

    fn key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn upsert(&self, conn: &rusqlite::Connection) {
        conn.execute("INSERT OR REPLACE INTO Track 
            (key, artist, album, title, liked) 
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (&self.key, &self.artist, &self.album, &self.title, &self.liked)).unwrap();
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone();
    }
}