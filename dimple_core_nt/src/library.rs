use std::{sync::Mutex, time::Duration};

use rusqlite::{backup::Backup, Connection, OptionalExtension};
use ulid::Generator;
use uuid::Uuid;

use crate::model::{ChangeLog, Playlist, Track};

pub struct Library {
    conn: Connection,
    ulids: Mutex<Generator>,
}

impl Library {
    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file.
    pub fn open(database_path: &str) -> Self {

        let conn = Connection::open(database_path).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS Metadata (
                key       TEXT PRIMARY KEY,
                value     TEXT
            );
            ",
            (),
        ).unwrap();

        conn.execute("
            INSERT INTO Metadata (key, value) VALUES ('library.uuid', ?1)
            ON CONFLICT DO NOTHING
            ",
            (Self::uuid_v4(),),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS Artist (
                key       TEXT PRIMARY KEY,
                name      TEXT
            );
            ",
            (),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS Track (
                key       TEXT PRIMARY KEY,
                artist    TEXT,
                album     TEXT,
                title     TEXT,
                path      TEXT NOT NULL,
                liked     BOOL NOT NULL DEFAULT false,
                UNIQUE (path)
            );
            ",
            (),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS Playlist (
                key       TEXT PRIMARY KEY,
                name      TEXT
            );
            ",
            (),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS PlaylistItem (
                key          TEXT PRIMARY KEY,
                Playlist_key TEXT NOT NULL,
                Track_key    TEXT NOT NULL,
                FOREIGN KEY (Playlist_key) REFERENCES Playlist(key),
                FOREIGN KEY (Track_key) REFERENCES Track(key)
            );
            ",
            (),
        ).unwrap();

        // A row is stored in the ChangeLog every time a change to a tracked
        // model is made. Each row gets a timestamp from a hybrid logical
        // clock which ensures the value is always increasing and that
        // the timestamps are mostly in wall time order.
        conn.execute("
            CREATE TABLE IF NOT EXISTS ChangeLog (
                actor     TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                model     TEXT NOT NULL,
                key       TEXT NOT NULL,
                op        TEXT NOT NULL,
                field     TEXT,
                value     TEXT,
                PRIMARY KEY (actor, timestamp)
            );
            ",
            (),
        ).unwrap();

        let library = Library {
            conn,
            ulids: Mutex::new(Generator::new()),
        };

        library
    }

    /// Returns the unique, permanent ID of this Library. This is created when
    /// the Library is created and doesn't change.
    pub fn uuid(&self) -> String {
        self.conn.query_row("SELECT value FROM Metadata WHERE key = 'library.uuid'", 
            (), 
            |row| {
                let s: String = row.get(0).unwrap();
                Ok(s)
            }).unwrap()
    }

    /// Backup this library to the specified path.
    pub fn backup(&self, output_path: &str) {
        let mut dst = Connection::open(output_path).unwrap();
        let backup = Backup::new(&self.conn, &mut dst).unwrap();
        backup.run_to_completion(250, Duration::from_millis(10), None).unwrap();
    }

    /// Import MediaFiles into the Library, creating or updating Tracks.
    pub fn import(&self, media_files: &[crate::scanner::media_file::MediaFile]) {
        for mf in media_files {
            let artist = mf.tag(symphonia::core::meta::StandardTagKey::Artist);
            let album = mf.tag(symphonia::core::meta::StandardTagKey::Album);
            let title = mf.tag(symphonia::core::meta::StandardTagKey::TrackTitle);
            let path = &mf.path;
            let mut track = self.track_by_path(path).or_else(|| Some(Track::default())).unwrap();
            track.artist = artist;
            track.album = album;
            track.title = title;
            track.path = path.to_owned();
            track.save(self);
        }
    }

    pub fn uuid_v4() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generates a ulid that is guaranteed to be monotonic.
    pub fn ulid(&self) -> String {
        self.ulids.lock().unwrap().generate().unwrap().to_string()
    }

    pub fn changelogs(&self) -> Vec<ChangeLog> {
        let mut stmt = self.conn.prepare("SELECT 
            actor, timestamp, model, key, op, field, value
            FROM ChangeLog ORDER BY timestamp ASC").unwrap();
        stmt.query_map([], |row| {
            Ok(ChangeLog {
                actor: row.get(0).unwrap(),
                timestamp: row.get(1).unwrap(),
                model: row.get(2).unwrap(),
                key: row.get(3).unwrap(),
                op: row.get(4).unwrap(),
                field: row.get(5).unwrap(),
                value: row.get(6).unwrap(),
            })
        })
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
    }

    pub fn find_newest_changelog_by_field(&self, model: &str, key: &str, field: &str) -> Option<ChangeLog> {
        self.conn.query_row_and_then("SELECT 
            actor, timestamp, model, key, op, field, value
            FROM ChangeLog 
            WHERE model = ?1 AND key = ?2 AND field = ?3
            ORDER BY timestamp DESC", 
            (model, key, field), |row| {
                Ok(ChangeLog {
                    actor: row.get(0).unwrap(),
                    timestamp: row.get(1).unwrap(),
                    model: row.get(2).unwrap(),
                    key: row.get(3).unwrap(),
                    op: row.get(4).unwrap(),
                    field: row.get(5).unwrap(),
                    value: row.get(6).unwrap(),
                })
            }).optional().unwrap()
    }

    pub fn tracks(&self) -> Vec<Track> {
        let mut stmt = self.conn.prepare("SELECT 
            key, artist, album, title, path, liked
            FROM Track
            ORDER BY artist, album, title").unwrap();
        stmt.query_map([], |row| {
            Ok(Track {
                key: row.get(0)?,
                artist: row.get(1)?,
                album: row.get(2)?,
                title: row.get(3)?,
                path: row.get(4)?,
                liked: row.get(5)?,
            })
        })
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
    }

    pub fn playlist_add(&self, playlist: &Playlist, track_key: &str) {
        self.conn.execute("INSERT INTO PlaylistItem 
            (key, Playlist_key, Track_key) 
            VALUES (?1, ?2, ?3)",
            (&Self::uuid_v4(), playlist.key.clone().unwrap(), track_key)).unwrap();
    }

    pub fn playlist_clear(&self, playlist: &Playlist) {
        self.conn.execute("DELETE FROM PlaylistItem
            WHERE Playlist_key = ?1", (playlist.key.clone().unwrap(),)).unwrap();
    }    

    pub fn track_by_path(&self, path: &str) -> Option<Track> {
        self.conn.query_row_and_then("SELECT 
            key, artist, album, title, path, liked
            FROM Track
            WHERE path = ?1", 
            (path,), |row| {
                Ok(Track {
                    key: row.get(0).unwrap(),
                    artist: row.get(1).unwrap(),
                    album: row.get(2).unwrap(),
                    title: row.get(3).unwrap(),
                    path: row.get(4).unwrap(),
                    liked: row.get(5).unwrap()
                })
            }).optional().unwrap()
    }
}

pub trait LibraryModel {
    fn save(&self, library: &Library) -> Self;
    fn get(library: &Library, key: &str) -> Option<Self> where Self: Sized;
    fn diff(&self, other: &Self) -> Vec<ChangeLog>;
    fn apply_diff(&mut self, diff: &[ChangeLog]);
}

impl LibraryModel for Track {
    fn save(&self, library: &Library) -> Self {
        // TODO txn
        // TODO this moves to library I think as generic save()
        let old = self.key.as_ref().and_then(|key| Self::get(library, &key))
            .or_else(|| Some(Track::default())).unwrap();
        let key = self.key.clone().or_else(|| Some(Library::uuid_v4()));
        library.conn.execute("INSERT OR REPLACE INTO Track 
            (key, artist, album, title, path, liked) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&key, &self.artist, &self.album, &self.title, &self.path, &self.liked)).unwrap();
        let new = Self::get(library, &key.unwrap()).unwrap();
        let diff = old.diff(&new);
        for mut change in diff {
            change.timestamp = library.ulid();
            change.actor = library.uuid();
            change.key = new.key.clone().unwrap();
            change.save(library);
        }
        // maybe like library.notify(diff)
        new
    }

    fn get(library: &Library, key: &str) -> Option<Self> {
        library.conn.query_row("SELECT key, artist, album, title, path, liked 
            FROM Track WHERE key = ?1", 
            (key,), 
            |row| Ok(Track {
                key: row.get(0)?,
                artist: row.get(1)?,
                album: row.get(2)?,
                title: row.get(3)?,
                path: row.get(4)?,
                liked: row.get(5)?,
            })).optional().unwrap()
    }
    
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
        if self.path != other.path {
            diff.push(ChangeLog { model: "Track".to_string(), 
                op: "set".to_string(), field: Some("path".to_string()), 
                value: Some(other.path.clone()), ..Default::default() });
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
                    if &field == "path" {
                        self.path = change.value.clone().unwrap();
                    }
                    if &field == "liked" {
                        self.liked = change.value.clone() == Some("true".to_string());
                    }
                }
            }
        }
    }    
}

impl LibraryModel for Playlist {
    fn save(&self, library: &Library) -> Self {
        let key = self.key.clone().or_else(|| Some(Library::uuid_v4()));
        let name = &self.name;
        library.conn.execute("INSERT OR REPLACE INTO Playlist 
            (key, name) 
            VALUES 
            (?1, ?2)",
            (&key, name)).unwrap();
            Self::get(library, &key.unwrap()).unwrap()
    }

    fn get(library: &Library, key: &str) -> Option<Self> where Self: Sized {
        let mut playlist = Playlist::default();
        let mut stmt = library.conn.prepare("SELECT
            Playlist.key, Playlist.name, 
            Track.key, Track.artist, Track.album, Track.title, Track.path, Track.liked
            FROM Playlist 
            LEFT JOIN PlaylistItem ON (PlaylistItem.playlist_key = Playlist.key)
            LEFT JOIN Track ON (Track.key = PlaylistItem.Track_key)
            WHERE PlayList.key = ?1").unwrap();
        let mut rows = stmt.query((key,)).unwrap();
        while let Some(row) = rows.next().unwrap() {
            playlist.key = row.get(0).unwrap();
            playlist.name = row.get(1).unwrap();
            if row.get::<_, Option<String>>(2).unwrap().is_some() {
                playlist.tracks.push(Track {
                    key: row.get(2).unwrap(),
                    artist: row.get(3).unwrap(),
                    album: row.get(4).unwrap(),
                    title: row.get(5).unwrap(),
                    path: row.get(6).unwrap(),
                    liked: row.get(7).unwrap(),
                });
            }
        }
        match playlist.key {
            None => None,
            Some(_) => Some(playlist),
        }
    }
    
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        todo!()
    }
    
    fn apply_diff(&mut self, diff: &[ChangeLog]) {
        todo!()
    }    
}

impl LibraryModel for ChangeLog {
    fn save(&self, library: &Library) -> Self {
        library.conn.execute("INSERT INTO ChangeLog         
            (actor, timestamp, model, key, op, field, value)
            VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT DO NOTHING",
            (&self.actor, &self.timestamp, &self.model, &self.key, &self.op, &self.field, &self.value)).unwrap();
        self.clone()
    }

    fn get(library: &Library, key: &str) -> Option<Self> {
        todo!()
    }
    
    fn diff(&self, other: &Self) -> Vec<ChangeLog> {
        todo!()
    }
    
    fn apply_diff(&mut self, diff: &[ChangeLog]) {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::model::Track;

    use super::{Library, LibraryModel};

    #[test]
    fn it_works() {
        let _library = Library::open(":memory:");
    }

    #[test]
    fn changelogs() {
        let library = Library::open(":memory:");
        let mut track = Track {
            artist: Some("Which Who".to_string()),
            title: Some("We All Eat Food".to_string()),
            ..Default::default()
        }.save(&library);
        track.artist = Some("The The".to_string());
        track.album = Some("Some Kind of Something".to_string());
        track.liked = true;
        track.save(&library);
        let changelogs = library.changelogs();
        assert!(changelogs.len() == 5);        
        for changelog in changelogs {
            println!("{:?}", changelog);
        }
    }

    #[test]
    fn diff() {
        let track = Track {
            artist: Some("The Newbs".to_string()),
            album: Some("Brand News".to_string()),
            title: Some("Fresh Stuff".to_string()),
            ..Default::default()
        };
        let diff = Track::default().diff(&track);
        let mut track2 = Track::default();
        track2.apply_diff(&diff);
        assert!(track == track2);
    }
}
