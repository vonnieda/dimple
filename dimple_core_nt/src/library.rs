use std::time::Duration;

use rusqlite::{backup::Backup, Connection, OptionalExtension};
use uuid::Uuid;

use crate::model::{Artist, ChangeLog, Playlist, Track};

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
                path      TEXT,
                liked     BOOL
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
                value     TEXT
            );
            ",
            (),
        ).unwrap();

        let library = Library {
            conn,
        };

        library.create_changelog_triggers("Artist");
        library.create_changelog_triggers("Track");
        library.create_changelog_triggers("Playlist");
        library.create_changelog_triggers("PlaylistItem");

        library
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
            key, artist, album, title, path, liked
            FROM Track").unwrap();
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

    fn create_changelog_triggers(&self, table_name: &str) {
        let column_names = self.get_column_names(table_name);

        // Create SQL fragments to record set_field ops when inserting any
        // initial values. Used in the insert trigger below.
        let mut sql_fragments = vec![];
        for column_name in column_names.clone() {
            sql_fragments.push(format!("
                    INSERT INTO ChangeLog 
                    (actor, timestamp, model, key, op, field, value)
                    VALUES (
                        (SELECT value FROM Metadata WHERE key = 'library.uuid'),
                        unixepoch('now', 'subsec'), 
                        '{table_name}', 
                        NEW.key, 
                        'insert_field', 
                        '{column_name}', 
                        NEW.{column_name}
                    );
            "));
        }
        let sql_fragments = sql_fragments.join("");

        // Create the Table_insert trigger. When a new object is inserted into
        // the table this will record an insert op with the new key, along with
        // one insert_field op per column with that column's new value.
        let trigger_name = format!("{}_insert", table_name);
        let sql = format!("
            CREATE TRIGGER IF NOT EXISTS {trigger_name} 
            AFTER INSERT ON {table_name}
            BEGIN
                INSERT INTO ChangeLog 
                (actor, timestamp, model, key, op)
                VALUES (
                    (SELECT value FROM Metadata WHERE key = 'library.uuid'),
                    unixepoch('now', 'subsec'), 
                    '{table_name}', 
                    NEW.key, 
                    'insert'
                );
                {sql_fragments}
            END;
        ");
        self.conn.execute(&sql, ()).unwrap();

        // Create Table_update_column triggers, one for each column. These
        // record a set_field op for each modified column.
        for column_name in column_names {
            let trigger_name = format!("{}_update_{}", table_name, column_name);
            let sql = format!("
                CREATE TRIGGER IF NOT EXISTS {trigger_name} 
                AFTER UPDATE OF {column_name} ON {table_name}
                -- Filters out updates that don't actually change the value.
                WHEN NEW.{column_name} != OLD.{column_name} 
                    OR NEW.{column_name} IS NULL OR OLD.{column_name} IS NULL
                BEGIN
                    INSERT INTO ChangeLog 
                    (actor, timestamp, model, key, op, field, value)
                    VALUES (
                        (SELECT value FROM Metadata WHERE key = 'library.uuid'),
                        unixepoch('now', 'subsec'), 
                        '{table_name}', 
                        OLD.key, 
                        'set_field', 
                        '{column_name}', 
                        NEW.{column_name}
                    );
                END;
            ");
            self.conn.execute(&sql, ()).unwrap();
        }
    }
}

pub trait LibraryModel {
    fn save(&self, library: &Library) -> Self;
    fn get(library: &Library, key: &str) -> Option<Self> where Self: Sized;
}

impl LibraryModel for Track {
    fn save(&self, library: &Library) -> Self {
        let key = self.key.clone().or_else(|| Some(Library::uuid_v4()));
        library.conn.execute("INSERT OR REPLACE INTO Track 
            (key, artist, album, title, path, liked) 
            VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6)",
            (&key, &self.artist, &self.album, &self.title, &self.path, &self.liked)).unwrap();
        Self::get(library, &key.unwrap()).unwrap()
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
                    liked: row.get(7).unwrap(),
                });
            }
        }
        match playlist.key {
            None => None,
            Some(_) => Some(playlist),
        }
    }
}

impl LibraryModel for ChangeLog {
    fn save(&self, library: &Library) -> Self {
        library.conn.execute("INSERT INTO ChangeLog 
            (actor, timestamp, model, key, op, field, value)
            VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (&self.actor, &self.timestamp, &self.model, &self.key, &self.op, &self.field, &self.value)).unwrap();
        self.clone()
    }

    fn get(library: &Library, key: &str) -> Option<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Track;

    use super::{Library, LibraryModel};

    #[test]
    fn it_works() {
        let library = Library::open(":memory:");
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
        track.save(&library);
        
        let changelogs = library.changelogs();
        assert!(changelogs.len() == 8);
        for changelog in changelogs {
            println!("{:?}", changelog);
        }
    }
}
