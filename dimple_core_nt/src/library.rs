use std::{sync::Mutex, time::Duration};

use rusqlite::{backup::Backup, Connection, OptionalExtension};
use symphonia::core::meta::StandardTagKey;
use ulid::Generator;
use uuid::Uuid;

use crate::model::{ChangeLog, Diff, FromRow, MediaFile, Model, Playlist, Track};

pub struct Library {
    pub conn: Connection,
    ulids: Mutex<Generator>,
}

impl Library {
    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file.
    pub fn open(database_path: &str) -> Self {

        let conn = Connection::open(database_path).unwrap();

        // TODO https://github.com/cljoly/rusqlite_migration/blob/master/examples/from-directory/src/main.rs
        let schema = include_str!("migrations/202411070001_initial.sql");

        conn.execute_batch(schema).unwrap();

        conn.execute("
            INSERT INTO Metadata (key, value) VALUES ('library.uuid', ?1)
            ON CONFLICT DO NOTHING
            ",
            (Uuid::new_v4().to_string(),),
        ).unwrap();

        let library = Library {
            conn,
            ulids: Mutex::new(Generator::new()),
        };

        library
    }

    /// Returns the unique, permanent ID of this Library. This is created when
    /// the Library is created and doesn't change.
    pub fn id(&self) -> String {
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

    /// Import MediaFiles into the Library, creating or updating Tracks and
    /// TrackSources.
    pub fn import(&self, input: &[crate::scanner::media_file::MediaFile]) {
        for input_mf in input {
            let artist = input_mf.tag(StandardTagKey::Artist);
            let album = input_mf.tag(StandardTagKey::Album);
            let title = input_mf.tag(StandardTagKey::TrackTitle);
            if artist.is_none() && album.is_none() && title.is_none() {
                println!("WARNING: Empty track info. Skipping {}.", input_mf.path.to_string());
                continue;
            }
            let file_path = &input_mf.path;
            let mut mf = self.find_media_file_by_file_path(file_path)
                .or_else(|| Some(MediaFile::default()))
                .unwrap();
            mf.file_path = file_path.to_owned();
            mf.artist = artist;
            mf.album = album;
            mf.title = title;
            self.save(&mf);
        }

        self.post_import_update_tracks();
    }

    fn post_import_update_tracks(&self) {
        for mf in self.list::<MediaFile>() {
            // TODO txn
            if self.find_track_for_media_file(&mf).is_none() {
                let track = Track {
                    artist: mf.artist,
                    album: mf.album,
                    title: mf.title,
                    ..Default::default()
                };
                self.save(&track);
            }
        }
    }

    /// Generates a ulid that is guaranteed to be monotonic.
    pub fn ulid(&self) -> String {
        self.ulids.lock().unwrap().generate().unwrap().to_string()
    }

    pub fn save<T: Model>(&self, obj: &T) -> T {
        // TODO txn

        // get the old object by key if one exists
        let old: T = obj.key().as_ref().and_then(|key| self.get(&key))
            .or_else(|| Some(T::default())).unwrap();
        // get the key or create a new one
        let key = obj.key().or_else(|| Some(Uuid::new_v4().to_string()));
        // execute the insert
        let mut obj = obj.clone();
        obj.set_key(key.clone());
        obj.upsert(&self.conn);
        // load the newly inserted object
        let new: T = self.get(&key.unwrap()).unwrap();
        if T::log_changes() {
            // if we're logging changes, diff the old to the new
            let diff = old.diff(&new);
            for mut change in diff {
                // each change gets a new ulid, the library actor, a new key
                // and gets saved
                change.timestamp = self.ulid();
                change.actor = self.id();
                change.model_key = new.key().clone().unwrap();
                self.save(&change);
            }
        }
        // TODO maybe like library.notify(diff)
        new
    }

    pub fn save_unlogged<T: Model>(&self, obj: &T) -> T {
        // TODO txn

        // get the key or create a new one
        let key = obj.key().or_else(|| Some(Uuid::new_v4().to_string()));
        // execute the insert
        let mut obj = obj.clone();
        obj.set_key(key.clone());
        obj.upsert(&self.conn);
        // load the newly inserted object
        let new: T = self.get(&key.unwrap()).unwrap();
        // TODO maybe like library.notify(diff)
        new
    }

    pub fn get<T: Model>(&self, key: &str) -> Option<T> {
        let sql = format!("SELECT * FROM {} WHERE key = ?1", T::table_name());
        self.conn.query_row(&sql, (key,), 
            |row| Ok(T::from_row(row))).optional().unwrap()
    }

    pub fn list<T: Model>(&self) -> Vec<T> {
        let sql = format!("SELECT * FROM {}", T::table_name());
        self.conn.prepare(&sql).unwrap()
            .query_map((), |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect()
    }

    pub fn changelogs(&self) -> Vec<ChangeLog> {
        let mut stmt = self.conn.prepare("SELECT * FROM ChangeLog 
            ORDER BY timestamp ASC").unwrap();
        stmt.query_map([], |row| Ok(ChangeLog::from_row(row)))
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
    }

    pub fn tracks(&self) -> Vec<Track> {
        let mut stmt = self.conn.prepare("SELECT * FROM Track
            ORDER BY artist, album, title").unwrap();
        stmt.query_map([], |row| Ok(Track::from_row(row)))
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
    }

    pub fn playlist_add(&self, playlist: &Playlist, track_key: &str) {
        self.conn.execute("INSERT INTO PlaylistItem 
            (key, playlist_key, track_key) 
            VALUES (?1, ?2, ?3)",
            (&Uuid::new_v4().to_string(), playlist.key.clone().unwrap(), track_key)).unwrap();
    }

    pub fn playlist_clear(&self, playlist: &Playlist) {
        self.conn.execute("DELETE FROM PlaylistItem
            WHERE playlist_key = ?1", (playlist.key.clone().unwrap(),)).unwrap();
    }    

    pub fn find_newest_changelog_by_field(&self, model: &str, key: &str, field: &str) -> Option<ChangeLog> {
        self.conn.query_row_and_then("SELECT * FROM ChangeLog 
            WHERE model = ?1 AND key = ?2 AND field = ?3
            ORDER BY timestamp DESC", 
            (model, key, field), |row| Ok(ChangeLog::from_row(row))).optional().unwrap()
    }

    pub fn find_media_file_by_file_path(&self, file_path: &str) -> Option<MediaFile> {
        self.conn.query_row_and_then("SELECT * FROM MediaFile
            WHERE file_path = ?1", 
            (file_path,), |row| Ok(MediaFile::from_row(row)))
            .optional().unwrap()
    }

    pub fn find_track_for_media_file(&self, media_file: &MediaFile) -> Option<Track> {
        // TODO naive, just for testing.
        self.conn.query_row_and_then("SELECT * FROM Track
            WHERE artist = ?1 AND album = ?2 AND title = ?3", 
            (media_file.artist.clone(), media_file.album.clone(), media_file.title.clone()), |row| Ok(Track::from_row(row)))
            .optional().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{model::{Diff, MediaFile, Track}, scanner::Scanner};

    use super::Library;

    #[test]
    fn it_works() {
        let _library = Library::open(":memory:");
    }

    #[test]
    fn tracks() {
        let library = Library::open(":memory:");
        library.tracks();
    }

    #[test]
    fn changelogs() {
        let library = Library::open(":memory:");
        let track = Track {
            artist: Some("Which Who".to_string()),
            title: Some("We All Eat Food".to_string()),
            ..Default::default()
        };
        let mut track = library.save(&track);
        track.artist = Some("The The".to_string());
        track.album = Some("Some Kind of Something".to_string());
        track.liked = true;
        library.save(&track);
        let changelogs = library.changelogs();
        assert!(changelogs.len() == 6);        
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

    #[test]
    fn import() {
        let library = Library::open(":memory:");
        assert!(library.list::<MediaFile>().len() == 0);
        library.import(&Scanner::scan_directory("media_files"));
        let num_mediafiles = library.list::<MediaFile>().len();
        assert!(library.list::<MediaFile>().len() > 0);
        library.import(&Scanner::scan_directory("media_files"));
        assert!(library.list::<MediaFile>().len() == num_mediafiles);
    }
}
