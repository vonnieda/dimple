
use rusqlite::Connection;

use crate::model::Track;

pub struct Library {
    conn: Connection,
}

impl Library {
    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file.
    pub fn open(database_path: &str) -> Self {

        let conn = Connection::open(database_path).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Track (
                key       UUID PRIMARY KEY,
                artist    TEXT,
                album     TEXT,
                title     TEXT,
                path      TEXT NOT NULL UNIQUE
            )",
            (),
        ).unwrap();

        Library {
            conn,
        }
    }

    /// Import MediaFiles into the Library, creating or updating Tracks.
    pub fn import(&self, media_files: &[crate::scanner::media_file::MediaFile]) {
        for mf in media_files {
            let key = uuid::Uuid::new_v4().to_string();
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

    pub fn list_tracks(&self) -> Vec<Track> {
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
}

#[cfg(test)]
mod tests {
    use super::Library;

    #[test]
    fn basics() {
        let library = Library::open(":memory:");
        let tracks = library.list_tracks();
        dbg!(&tracks);
    }
}
