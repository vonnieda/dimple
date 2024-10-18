use std::time::Duration;

use rusqlite::{backup::Backup, Connection};

use crate::model::{Playlist, Track};

pub struct Library {
    conn: Connection,
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
            (Self::new_uuid(),),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS Track (
                key       UUID PRIMARY KEY,
                artist    TEXT,
                album     TEXT,
                title     TEXT,
                path      TEXT NOT NULL UNIQUE
            );
            ",
            (),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS Playlist (
                key       UUID PRIMARY KEY,
                name      TEXT
            );
            ",
            (),
        ).unwrap();

        conn.execute("
            CREATE TABLE IF NOT EXISTS PlaylistItem (
                key UUID PRIMARY KEY,
                Playlist_key UUID NOT NULL,
                Track_key UUID NOT NULL,
                FOREIGN KEY (Playlist_key) REFERENCES Playlist(key),
                FOREIGN KEY (Track_key) REFERENCES Track(key)
            );
            ",
            (),
        ).unwrap();

        Library {
            conn,
        }
    }

    /// Import MediaFiles into the Library, creating or updating Tracks.
    pub fn import(&self, media_files: &[crate::scanner::media_file::MediaFile]) {
        for mf in media_files {
            let key = Self::new_uuid();
            let artist = mf.tag(symphonia::core::meta::StandardTagKey::Artist);
            let album = mf.tag(symphonia::core::meta::StandardTagKey::Album);
            let title = mf.tag(symphonia::core::meta::StandardTagKey::TrackTitle);
            let path = &mf.path;
            self.conn.execute("INSERT INTO Track 
                (key, artist, album, title, path)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT (path)
                DO
                    UPDATE SET
                    artist = excluded.artist,
                    album = excluded.album,
                    title = excluded.title",
                (key, artist, album, title, path)).unwrap();
        }
    }

    pub fn tracks(&self) -> Vec<Track> {
        let mut stmt = self.conn.prepare("SELECT 
            key, artist, album, title, path 
            FROM Track").unwrap();
        stmt.query_map([], |row| {
            Ok(Track {
                key: row.get(0)?,
                artist: row.get(1)?,
                album: row.get(2)?,
                title: row.get(3)?,
                path: row.get(4)?,
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
            (&Self::new_uuid(), playlist.key.clone().unwrap(), track_key)).unwrap();
    }

    pub fn playlist_clear(&self, playlist: &Playlist) {
        self.conn.execute("DELETE FROM PlaylistItem
            WHERE Playlist_key = ?1", (playlist.key.clone().unwrap(),)).unwrap();
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

    fn playlist_by_key(&self, key: &str) -> Option<Playlist> {
        let mut playlist = Playlist::default();
        let mut stmt = self.conn.prepare("SELECT
            Playlist.key, Playlist.name, 
            Track.key, Track.artist, Track.album, Track.title, Track.path
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
                });
            }
        }
        match playlist.key {
            None => None,
            Some(_) => Some(playlist),
        }
    }

    fn create_or_update_playlist(&self, playlist: &Playlist) -> Playlist {
        let key = playlist.key.clone().or_else(|| Some(Self::new_uuid()));
        let name = &playlist.name;
        self.conn.execute("INSERT OR REPLACE INTO Playlist 
            (key, name) 
            VALUES 
            (?1, ?2)",
            (&key, name)).unwrap();
        self.playlist_by_key(&key.unwrap()).unwrap()
    }

    pub fn get_or_create_playlist_by_key(&self, key: &str) -> Playlist {
        self.playlist_by_key(key).or_else(|| Some(self.create_or_update_playlist(&Playlist { 
            key: Some(key.to_string()), 
            ..Default::default()
        }))).unwrap()
    }

    pub fn backup(&self, output_path: &str) {
        let mut dst = Connection::open(output_path).unwrap();
        let backup = Backup::new(&self.conn, &mut dst).unwrap();
        backup.run_to_completion(250, Duration::from_millis(10), None).unwrap();
    }

    fn new_uuid() -> String {
        return uuid::Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Library;

    #[test]
    fn basics() {
        // let library = Library::open(":memory:");
        // let mut playlist = library.play_queue();
        // assert!(playlist.key == Some("__dimple_system_play_queue".to_string()));
        // assert!(playlist.name.is_none());
        // assert!(playlist.tracks.len() == 0);
        // playlist.name = Some("Dimple System Play Queue".to_string());
        // library.create_or_update_playlist(&playlist);
        // let playlist = library.get_or_create_playlist_by_key("__dimple_system_play_queue");
        // assert!(playlist.name == Some("Dimple System Play Queue".to_string()));
    }
}
