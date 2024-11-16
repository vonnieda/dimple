use std::{sync::{Arc, Mutex, RwLock}, time::{Duration, Instant}};

use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rusqlite::{backup::Backup, Connection, OptionalExtension};
use symphonia::core::meta::StandardTagKey;
use ulid::Generator;
use uuid::Uuid;

use crate::{model::{Blob, ChangeLog, FromRow, MediaFile, Model, Playlist, Track, TrackSource}, scanner::media_file::ScannedFile, sync::Sync};

#[derive(Clone)]
pub struct Library {
    _conn: Arc<Mutex<Connection>>,
    database_path: String,
    ulids: Arc<Mutex<Generator>>,
    synchronizers: Arc<RwLock<Vec<Sync>>>,
}

/// TODO change notifications
/// TODO start changing to Release and friends to easier port to GUI
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
            _conn: Arc::new(Mutex::new(conn)),
            database_path: database_path.to_string(),
            ulids: Arc::new(Mutex::new(Generator::new())),
            synchronizers: Arc::new(RwLock::new(vec![])),
        };

        library
    }

    pub fn conn(&self) -> Connection {
        Connection::open(self.database_path.clone()).unwrap()
    }

    /// Returns the unique, permanent ID of this Library. This is created when
    /// the Library is created and doesn't change.
    pub fn id(&self) -> String {
        self.conn().query_row("SELECT value FROM Metadata WHERE key = 'library.uuid'", 
            (), 
            |row| {
                let s: String = row.get(0).unwrap();
                Ok(s)
            }).unwrap()
    }

    /// Backup this library to the specified path.
    pub fn backup(&self, output_path: &str) {
        let mut dst = Connection::open(output_path).unwrap();
        let src = self.conn();
        let backup = Backup::new(&src, &mut dst).unwrap();
        backup.run_to_completion(250, Duration::from_millis(10), None).unwrap();
    }

    /// Import MediaFiles into the Library, creating or updating Tracks,
    /// TrackSources, Blobs, etc.
    /// TODO okay this is slow cause we are scanning all the files first no
    /// matter what, reading all their tags and images and shit, and we might
    /// just ignore that file based on it's sha, so fix that.
    pub fn import(&self, input: &[crate::scanner::media_file::ScannedFile]) {
        let library = self.clone();
        // TODO getting a lot of "database table is locked: ChangeLog" when using par_iter
        input.par_iter().for_each(|input| {
            library.import_internal(input);
        });
    }

    fn import_internal(&self, input: &ScannedFile) {
        // TODO txn
        let file_path = std::fs::canonicalize(&input.path).unwrap();
        let file_path = file_path.to_str().unwrap();

        let blob = Blob::read(file_path);
        let blob = self.find_blob_by_sha256(&blob.sha256)
            .or_else(|| Some(self.save(&blob)))
            .unwrap();

        let media_file = self.find_media_file_by_file_path(file_path)
            .or_else(|| Some(self.save(&MediaFile {
                file_path: file_path.to_owned(),
                sha256: blob.sha256.clone(),
                artist: input.tag(StandardTagKey::Artist),
                album: input.tag(StandardTagKey::Album),
                title: input.tag(StandardTagKey::TrackTitle),
                ..Default::default()
            })))
            .unwrap();

        if self.track_sources_by_blob(&blob).is_empty() {
            // TODO temp, eventually uses more matching
            // or maybe just always create and de-dupe?
            let track = self.find_track_for_media_file(&media_file)
                .or_else(|| Some(self.save(&Track {
                    artist: media_file.artist,
                    album: media_file.album,
                    title: media_file.title,
                    ..Default::default()
                })))
                .unwrap();

            let _source = self.save(&TrackSource {
                track_key: track.key.unwrap(),
                blob_key: blob.key.clone(),
                ..Default::default()
            });
        }
    }

    pub fn add_sync(&self, sync: Sync) {
        self.synchronizers.write().unwrap().push(sync);
    }

    pub fn sync(&self) {
        if let Ok(syncs) = self.synchronizers.read() {
            for sync in syncs.iter() {
                sync.sync(self);
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
        obj.upsert(&self.conn());
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

    /// TODO I think drop Model and use a trait that excludes Diff and such
    /// to make this more clear. And then I think I can drop Model.log_changes
    pub fn save_unlogged<T: Model>(&self, obj: &T) -> T {
        // TODO txn

        // get the key or create a new one
        let key = obj.key().or_else(|| Some(Uuid::new_v4().to_string()));
        // execute the insert
        let mut obj = obj.clone();
        obj.set_key(key.clone());
        obj.upsert(&self.conn());
        // load the newly inserted object
        let new: T = self.get(&key.unwrap()).unwrap();
        // TODO maybe like library.notify(diff)
        new
    }

    pub fn get<T: Model>(&self, key: &str) -> Option<T> {
        let sql = format!("SELECT * FROM {} WHERE key = ?1", T::table_name());
        self.conn().query_row(&sql, (key,), 
            |row| Ok(T::from_row(row))).optional().unwrap()
    }

    pub fn list<T: Model>(&self) -> Vec<T> {
        let sql = format!("SELECT * FROM {}", T::table_name());
        self.conn().prepare(&sql).unwrap()
            .query_map((), |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect()
    }

    pub fn query<T: Model>(&self, sql: &str) -> Vec<T> {
        let conn = self.conn();
        let result = conn.prepare(&sql).unwrap()
            .query_map((), |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect();
        result
    }

    pub fn changelogs(&self) -> Vec<ChangeLog> {
        self.query("SELECT * FROM ChangeLog ORDER BY timestamp ASC")
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.query("SELECT * FROM Track ORDER BY artist, album, title")
    }

    pub fn playlist_add(&self, playlist: &Playlist, track_key: &str) {
        self.conn().execute("INSERT INTO PlaylistItem 
            (key, playlist_key, track_key) 
            VALUES (?1, ?2, ?3)",
            (&Uuid::new_v4().to_string(), playlist.key.clone().unwrap(), track_key)).unwrap();
    }

    pub fn playlist_clear(&self, playlist: &Playlist) {
        self.conn().execute("DELETE FROM PlaylistItem
            WHERE playlist_key = ?1", (playlist.key.clone().unwrap(),)).unwrap();
    }    

    pub fn find_newest_changelog_by_field(&self, model: &str, model_key: &str, field: &str) -> Option<ChangeLog> {
        self.conn().query_row_and_then("SELECT * FROM ChangeLog 
            WHERE model = ?1 AND model_key = ?2 AND field = ?3
            ORDER BY timestamp DESC", 
            (model, model_key, field), |row| Ok(ChangeLog::from_row(row))).optional().unwrap()
    }

    pub fn find_media_file_by_file_path(&self, file_path: &str) -> Option<MediaFile> {
        self.conn().query_row_and_then("SELECT * FROM MediaFile
            WHERE file_path = ?1", 
            (file_path,), |row| Ok(MediaFile::from_row(row)))
            .optional().unwrap()
    }

    pub fn find_blob_by_sha256(&self, sha256: &str) -> Option<Blob> {
        self.conn().query_row_and_then("SELECT * FROM Blob
            WHERE sha256 = ?1", 
            (sha256,), |row| Ok(Blob::from_row(row)))
            .optional().unwrap()
    }

    pub fn find_track_for_media_file(&self, media_file: &MediaFile) -> Option<Track> {
        // TODO naive, just for testing.
        self.conn().query_row_and_then("SELECT * FROM Track
            WHERE artist = ?1 AND album = ?2 AND title = ?3", 
            (media_file.artist.clone(), media_file.album.clone(), media_file.title.clone()), |row| Ok(Track::from_row(row)))
            .optional().unwrap()
    }

    pub fn track_sources_for_track(&self, track: &Track) -> Vec<TrackSource> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM TrackSource
            WHERE track_key = ?1").unwrap();
        stmt.query_map([track.key.clone()], |row| Ok(TrackSource::from_row(row)))
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
    }
        
    pub fn track_sources_by_blob(&self, blob: &Blob) -> Vec<TrackSource> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM TrackSource
            WHERE blob_key = ?1").unwrap();
        stmt.query_map([blob.key.clone()], |row| Ok(TrackSource::from_row(row)))
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
    }

    pub fn media_files_by_sha256(&self, sha256: &str) -> Vec<MediaFile> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM MediaFile
            WHERE sha256 = ?1").unwrap();
        stmt.query_map([sha256], |row| Ok(MediaFile::from_row(row)))
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
    }

    pub fn load_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
        for media_file in self.media_files_by_sha256(&blob.sha256) {
            if let Ok(content) = std::fs::read(&media_file.file_path) {
                info!("Found blob sha256 {} at {}", blob.sha256, &media_file.file_path);
                return Some(content)
            }
        }
        for sync in self.synchronizers.read().unwrap().iter() {
            if let Some(content) = sync.load_blob_content(blob) {
                info!("Found blob sha256 {} in sync", blob.sha256);
                return Some(content)
            }
        }
        None
    }

    pub fn load_local_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
        for media_file in self.media_files_by_sha256(&blob.sha256) {
            if let Ok(content) = std::fs::read(media_file.file_path) {
                return Some(content)
            }
        }
        None
    }

    pub fn load_track_content(&self, track: &Track) -> Option<Vec<u8>> {
        for source in self.track_sources_for_track(track) {
            if let Some(blob_key) = source.blob_key {
                if let Some(blob) = self.get::<Blob>(&blob_key) {
                    if let Some(content) = self.load_blob_content(&blob) {
                        return Some(content)
                    }
                }
            }
        }
        None
    }

    /// Test that the database matches the combined state of the changelog.
    pub fn verify() {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{model::{Diff, MediaFile, Track}, scanner::Scanner};

    use super::Library;

    #[test]
    fn it_works() {
        let _library = Library::open("file:3cd2ed69-2945-49db-b7b9-bdf1d4f464d8?mode=memory&cache=shared");
    }

    #[test]
    fn tracks() {
        let library = Library::open("file:18de25eb-5ffb-4351-bce6-14969f5293e4?mode=memory&cache=shared");
        library.tracks();
    }

    #[test]
    fn changelogs() {
        let library = Library::open("file:7f59f615-f828-4db9-a5b2-8ae6ee4b4e2f?mode=memory&cache=shared");
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
        let library = Library::open("file:6384d9e0-74c1-4ecd-9ea3-b5d0118f134e?mode=memory&cache=shared");
        assert!(library.list::<MediaFile>().len() == 0);
        let media_files = Scanner::scan_directory("media_files_small");
        assert!(media_files.len() > 0);
        library.import(&media_files);
        let num_mediafiles = library.list::<MediaFile>().len();
        assert!(library.list::<MediaFile>().len() > 0);
        library.import(&Scanner::scan_directory("media_files_small"));
        assert!(library.list::<MediaFile>().len() == num_mediafiles);
    }

    #[test]
    fn load_track_content() {
        let library = Library::open("file:6384d9e0-74c1-4e1d-9ea3-b5d0198f134e?mode=memory&cache=shared");
        library.import(&Scanner::scan_directory("media_files_small"));
        let track = &library.tracks()[0];
        let content = library.load_track_content(track).unwrap();
        assert!(content.len() > 0);
    }
}
