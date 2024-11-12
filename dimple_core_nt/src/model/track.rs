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
        if self.key != other.key {
            diff.push(ChangeLog { model: "Track".to_string(), 
                op: "set".to_string(), field: Some("key".to_string()), 
                value: other.key.clone(), ..Default::default() });
        }
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
                    if &field == "key" {
                        self.key = change.value.clone();
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
    
    fn log_changes() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open(":memory:");
        let mut model = library.save(&Track::default());
        assert!(model.key.is_some());
        assert!(model.artist.is_none());
        model.artist = Some("Artist".to_string());
        let model = library.save(&model);
        let model: Track = library.get(&model.key.unwrap()).unwrap();
        assert!(model.artist == Some("Artist".to_string()));
    }

    #[test]
    fn diff() {
        let a = Track::default();
        let b = Track {
            key: Some("key".to_string()),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
            liked: true,
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 5);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("artist".to_string()));
        assert!(diff[2].field == Some("album".to_string()));
        assert!(diff[3].field == Some("title".to_string()));
        assert!(diff[4].field == Some("liked".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Track::default();
        let b = Track {
            key: Some("key".to_string()),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
            liked: true,
        };
        let diff = a.diff(&b);
        let mut c = Track::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}