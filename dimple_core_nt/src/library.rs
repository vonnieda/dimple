use std::time::Duration;

use rusqlite::{backup::Backup, Connection, OptionalExtension};
use uuid::Uuid;

use crate::model::{Artist, Playlist, Track};

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
            (Self::uuid_v4(),),
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
                key          UUID PRIMARY KEY,
                Playlist_key UUID NOT NULL,
                Track_key    UUID NOT NULL,
                FOREIGN KEY (Playlist_key) REFERENCES Playlist(key),
                FOREIGN KEY (Track_key) REFERENCES Track(key)
            );
            ",
            (),
        ).unwrap();
        //
        // A row is stored in the ChangeLog every time a change to a tracked
        // model is made. Each row gets a timestamp from a hybrid logical
        // clock which ensures the value is always increasing and that
        // the timestamps are roughly in wall time order. 
        // 
        // timestamp: From HLC
        // model:     Name of model type, e.g. Artist
        // key:       Key of the model being operated on.
        // op:        [set_field].
        // field:     (Optional) Name of field being set for set_field ops.
        // value:     (Optional) Value of field being set for set_field ops.
        // 
        conn.execute("
            CREATE TABLE IF NOT EXISTS ChangeLog (
                timestamp TEXT NOT NULL,
                model     TEXT NOT NULL,
                key       UUID NOT NULL,
                op        TEXT NOT NULL,
                field     TEXT,
                value     TEXT
            );
            ",
            (),
        ).unwrap();

        let library = Library {
            conn,
        };

        library.enable_changelog_tracking("Track");

        library
    }

    /// Import MediaFiles into the Library, creating or updating Tracks.
    pub fn import(&self, media_files: &[crate::scanner::media_file::MediaFile]) {
        for mf in media_files {
            let key = Self::uuid_v4();
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
            (&Self::uuid_v4(), playlist.key.clone().unwrap(), track_key)).unwrap();
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

    pub fn backup(&self, output_path: &str) {
        let mut dst = Connection::open(output_path).unwrap();
        let backup = Backup::new(&self.conn, &mut dst).unwrap();
        backup.run_to_completion(250, Duration::from_millis(10), None).unwrap();
    }

    pub fn uuid_v4() -> String {
        Uuid::new_v4().to_string()
    }

    fn enable_changelog_tracking(&self, table_name: &str) {
        // get the columns for the table
        let column_names = self.get_column_names(table_name);
        let trigger_name = format!("{}_insert", table_name);
        let sql = format!("
            CREATE TRIGGER IF NOT EXISTS {trigger_name} 
            INSERT ON {table_name}
            BEGIN
                INSERT INTO ChangeLog 
                (timestamp, model, key, op)
                VALUES (
                    unixepoch('now', 'subsec'), 
                    '{table_name}', 
                    NEW.key, 
                    'insert'
                );
            END;
        ");
        self.conn.execute(&sql, ()).unwrap();

        // I think it can be argued that doing all this generated SQL and
        // triggers is stupid when all the writes are supposed to go through
        // the save function anyway. And for that matter, the triggers are
        // going to cause problems when merging updates. So I think just make
        // save handle also writing to the ChangeLog, and then also that gives
        // me a good place to put the calls for observers. Which is what the
        // ChangeLog should be. Right.
        for column_name in column_names {
            let trigger_name = format!("{}_insert_{}", table_name, column_name);
            let sql = format!("
                CREATE TRIGGER IF NOT EXISTS {trigger_name} 
                UPDATE OF {column_name} ON {table_name}
                WHEN NEW.{column_name} != OLD.{column_name} 
                    OR NEW.{column_name} IS NULL OR OLD.{column_name} IS NULL
                BEGIN
                    INSERT INTO ChangeLog 
                    (timestamp, model, key, op, field, value)
                    VALUES (
                        unixepoch('now', 'subsec'), 
                        '{table_name}', 
                        OLD.key, 
                        'update', 
                        '{column_name}', 
                        NEW.{column_name}
                    );
                END;
            ");
            self.conn.execute(&sql, ()).unwrap();
        }

        for column_name in column_names {
            let trigger_name = format!("{}_update_{}", table_name, column_name);
            let sql = format!("
                CREATE TRIGGER IF NOT EXISTS {trigger_name} 
                UPDATE OF {column_name} ON {table_name}
                WHEN NEW.{column_name} != OLD.{column_name} 
                    OR NEW.{column_name} IS NULL OR OLD.{column_name} IS NULL
                BEGIN
                    INSERT INTO ChangeLog 
                    (timestamp, model, key, op, field, value)
                    VALUES (
                        unixepoch('now', 'subsec'), 
                        '{table_name}', 
                        OLD.key, 
                        'update', 
                        '{column_name}', 
                        NEW.{column_name}
                    );
                END;
            ");
            self.conn.execute(&sql, ()).unwrap();
        }
    }

    fn get_column_names(&self, table_name: &str) -> Vec<String> {
        let mut column_names = vec![];
        let mut stmt = self.conn.prepare("SELECT * FROM pragma_table_info(?1) AS tblInfo").unwrap();
        let mut rows = stmt.query((table_name,)).unwrap();
        while let Some(row) = rows.next().unwrap() {
            column_names.push(row.get(1).unwrap());
        }
        column_names
    }
}

pub trait LibraryModel {
    fn save(&self, library: &Library) -> Self;
    fn get(library: &Library, key: &str) -> Option<Self> where Self: Sized;
}

impl LibraryModel for Track {
    fn save(&self, library: &Library) -> Self {
        match &self.key {
            Some(key) => {
                library.conn.execute("
                    UPDATE Track 
                    SET artist = ?2, album = ?3, title = ?4, path = ?5
                    WHERE key = ?1
                    ",
                    (&key, &self.artist, &self.album, &self.title, &self.path)).unwrap();
                Self::get(library, &key).unwrap()
            },
            None => {
                let key = Library::uuid_v4();
                library.conn.execute("INSERT INTO Track 
                    (key, artist, album, title, path) 
                    VALUES 
                    (?1, ?2, ?3, ?4, ?5)",
                    (&key, &self.artist, &self.album, &self.title, &self.path)).unwrap();
                Self::get(library, &key).unwrap()
            }
        }
    }

    fn get(library: &Library, key: &str) -> Option<Self> {
        library.conn.query_row("SELECT key, artist, album, title, path 
            FROM Track WHERE key = ?1", 
            (key,), 
            |row| Ok(Track {
                key: row.get(0).unwrap(),
                artist: row.get(1).unwrap(),
                album: row.get(2).unwrap(),
                title: row.get(3).unwrap(),
                path: row.get(4).unwrap(),
            })).optional().unwrap()
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

    // TODO Okay I think this is a mistake! 
    fn get(library: &Library, key: &str) -> Option<Self> where Self: Sized {
        let mut playlist = Playlist::default();
        let mut stmt = library.conn.prepare("SELECT
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
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::model::Track;

    use super::{Library, LibraryModel};

    #[test]
    fn basics() {
        let library = Library::open(":memory:");
        let mut track = Track {
            artist: Some("Which Who".to_string()),
            title: Some("We All Eat Food".to_string()),
            ..Default::default()
        }.save(&library);
        track.artist = Some("The The".to_string());
        track.album = Some("Some Kind of Something".to_string());
        track.save(&library);

        let mut stmt = library.conn.prepare("SELECT * FROM ChangeLog").unwrap();
        let mut rows = stmt.query(()).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let timestamp: String = row.get(0).unwrap();
            let model: String = row.get(1).unwrap();
            let key: String = row.get(2).unwrap();
            let op: String = row.get(3).unwrap();
            let field: Option<String> = row.get(4).unwrap();
            let value: Option<String> = row.get(5).unwrap();
            println!("{timestamp} {model} {key} {op} {:?} {:?}", field, value);
        }
    }
}
