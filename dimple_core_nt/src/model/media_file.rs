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
            file_path: row.get("file_path").unwrap(),
            artist: row.get("artist").unwrap(),
            album: row.get("album").unwrap(),
            title: row.get("title").unwrap(),
        }
    }
}

impl Diff for MediaFile {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        let mut diff = vec![];
        if self.key != other.key {
            diff.push(ChangeLog { model: "TrackSource".to_string(), 
                op: "set".to_string(), field: Some("key".to_string()), 
                value: other.key.clone(), ..Default::default() });
        }
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
                    if &field == "key" {
                        self.key = change.value.clone();
                    }
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
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (&self.key, &self.artist, &self.album, &self.title, &self.file_path)).unwrap();
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::Diff};

    use super::MediaFile;

    #[test]
    fn library_crud() {
        let library = Library::open(":memory:");
        let mut model = library.save(&MediaFile::default(), true);
        assert!(model.key.is_some());
        assert!(model.artist.is_none());
        model.artist = Some("Artist".to_string());
        let model = library.save(&model, true);
        let model: MediaFile = library.get(&model.key.unwrap()).unwrap();
        assert!(model.artist == Some("Artist".to_string()));
    }

    #[test]
    fn diff() {
        let a = MediaFile::default();
        let b = MediaFile {
            key: Some("key".to_string()),
            file_path: "file_path".to_string(),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 5);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("file_path".to_string()));
        assert!(diff[2].field == Some("artist".to_string()));
        assert!(diff[3].field == Some("album".to_string()));
        assert!(diff[4].field == Some("title".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = MediaFile::default();
        let b = MediaFile {
            key: Some("key".to_string()),
            file_path: "file_path".to_string(),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 5);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("file_path".to_string()));
        assert!(diff[2].field == Some("artist".to_string()));
        assert!(diff[3].field == Some("album".to_string()));
        assert!(diff[4].field == Some("title".to_string()));

        let mut c = MediaFile::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}