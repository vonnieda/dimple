use rusqlite::Row;

use crate::library::Library;

use super::{ChangeLog, Diff, FromRow, Model, Track};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub tracks: Vec<Track>,
}

impl FromRow for Playlist {
    fn from_row(row: &Row) -> Self {
        Self {
            key: row.get("key").unwrap(),
            name: row.get("name").unwrap(),
            ..Default::default()
        }
    }
}

impl Diff for Playlist {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        let mut diff = vec![];
        if self.key != other.key {
            diff.push(ChangeLog { model: "Playlist".to_string(), 
                op: "set".to_string(), field: Some("key".to_string()), 
                value: other.key.clone(), ..Default::default() });
        }
        if self.name != other.name {
            diff.push(ChangeLog { model: "Playlist".to_string(), 
                op: "set".to_string(), field: Some("name".to_string()), 
                value: other.name.clone(), ..Default::default() });
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
                    if &field == "name" {
                        self.name = change.value.clone();
                    }
                }
            }
        }
    }    
}

impl Model for Playlist {
    fn table_name(&self) -> String {
        "Playlist".to_string()
    }

    fn key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn upsert(&self, conn: &rusqlite::Connection) {
        conn.execute("INSERT OR REPLACE INTO Playlist 
            (key, name) 
            VALUES (?1, ?2)",
            (&self.key, &self.name)).unwrap();
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone()
    }
        
    fn log_changes(&self) -> bool {
        true
    }

    fn hydrate(&mut self, library: &Library) {
        let sql = "
            SELECT Track.*
            FROM PlaylistItem
            JOIN Track ON (Track.key = PlaylistItem.Track_key)
            WHERE PlaylistItem.playlist_key = ?1
        ";
        self.tracks = library.query(sql, (self.key.clone(),));
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, Model, Playlist, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:e3b2df54-d10a-4530-b753-1fc82295ad32?mode=memory&cache=shared");
        let mut model = library.save(&Playlist::default());
        assert!(model.key.is_some());
        assert!(model.name.is_none());
        model.name = Some("name".to_string());
        let model = library.save(&model);
        let model: Playlist = library.get(&model.key.unwrap()).unwrap();
        assert!(model.name == Some("name".to_string()));
    }

    #[test]
    fn diff() {
        let a = Playlist::default();
        let b = Playlist {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            ..Default::default()
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 2);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("name".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Playlist::default();
        let b = Playlist {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            ..Default::default()
        };
        let diff = a.diff(&b);
        let mut c = Playlist::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }

    #[test]
    fn hydrate() {
        let library = Library::open("file:5d29e64a-5418-4773-b557-56448ae09efd?mode=memory&cache=shared");
        let playlist = library.save(&Playlist::default());
        for _ in 0..3 {
            let track = library.save(&Track::default());
            library.playlist_add(&playlist, &track.key.unwrap());
        }
        let mut playlist: Playlist = library.get(&playlist.key.unwrap()).unwrap();
        assert!(playlist.tracks.len() == 0);
        playlist.hydrate(&library);
        assert!(playlist.tracks.len() == 3);
    }
}