use std::{fmt::Debug, sync::{Arc, Mutex, RwLock}, time::Duration};

use image::DynamicImage;
use include_dir::{include_dir, Dir};
use log::info;
use r2d2::{CustomizeConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{backup::Backup, Connection, OptionalExtension, Params};
use rusqlite_migration::Migrations;
use threadpool::ThreadPool;
use ulid::Generator;
use uuid::Uuid;

use crate::{model::{Artist, Blob, ChangeLog, FromRow, Genre, LibraryModel, MediaFile, ModelBasics as _, Release, Track, TrackSource}, notifier::Notifier, sync::Sync};

#[derive(Clone)]
pub struct Library {
    pool: Pool<SqliteConnectionManager>,
    ulids: Arc<Mutex<Generator>>,
    // TODO I really think I want to get rid of this and put it somewhere
    // higher level. Note: Yea, it's going into Plugins and we're deleting
    // sync from Library entirely.
    synchronizers: Arc<RwLock<Vec<Sync>>>,
    notifier: Notifier<LibraryEvent>,
    threadpool: ThreadPool,
}

#[derive(Debug)]
struct LibraryConnectionCustomizer;
impl CustomizeConnection<rusqlite::Connection, rusqlite::Error> for LibraryConnectionCustomizer {
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(())
    }
}

impl Library {
    pub fn open_memory() -> Self {
        let manager = r2d2_sqlite::SqliteConnectionManager::memory();
        let pool = r2d2::Pool::builder()
            .max_size(1)
            .connection_customizer(Box::new(LibraryConnectionCustomizer{}))
            .build(manager)
            .unwrap();

        let library = Library {
            pool,
            ulids: Arc::new(Mutex::new(Generator::new())),
            synchronizers: Arc::new(RwLock::new(vec![])),
            notifier: Notifier::new(),
            threadpool: ThreadPool::new(1),
        };

        library.initialize_db();

        library
    }

    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file. If the directory does not exist
    /// it (and all parents) will be created.
    pub fn open(database_path: &str) -> Self {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(database_path);
        let pool = r2d2::Pool::builder()
            .max_size(24) // probably should be like num_cores * N but 24 feels nice
            .connection_customizer(Box::new(LibraryConnectionCustomizer{}))
            .build(manager)
            .unwrap();

        let library = Library {
            pool,
            ulids: Arc::new(Mutex::new(Generator::new())),
            synchronizers: Arc::new(RwLock::new(vec![])),
            notifier: Notifier::new(),
            threadpool: ThreadPool::new(1),
        };
        
        library.initialize_db();

        library
    }

    fn initialize_db(&self) {
        let mut conn = self.conn();

        static MIGRATION_DIR: Dir = include_dir!("./dimple_core/src/migrations");
        let migrations = Migrations::from_directory(&MIGRATION_DIR).unwrap();

        migrations.to_latest(&mut conn).unwrap();

        conn.execute("
            INSERT INTO Metadata (key, value) VALUES ('library.uuid', ?1)
            ON CONFLICT DO NOTHING
            ",
            (Uuid::new_v4().to_string(),),
        ).unwrap();
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
    /// TrackSources, Blobs, etc. path can be either a file or directory. If
    /// it is a directory it will be recursively scanned.
    /// TODO this goes away and into plugins too, I think.
    pub fn import(&self, path: &str) {
        crate::import::import(self, path);
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

    pub fn on_change(&self, l: Box<dyn Fn(&LibraryEvent) + Send>) {
        self.notifier.on_notify(l);
    }

    fn emit_change(&self, type_name: &str, key: &str) {
        let notifier = self.notifier.clone();
        let event = LibraryEvent {
            library: self.clone(),
            type_name: type_name.to_string(),
            key: key.to_string(),
        };
        self.threadpool.execute(move || {
            notifier.notify(&event);
        });
    }

    /// Generates a ulid that is guaranteed to be monotonic.
    pub fn ulid(&self) -> String {
        self.ulids.lock().unwrap().generate().unwrap().to_string()
    }

    pub fn insert<T: LibraryModel>(&self, obj: &T) -> T {
        let conn = self.conn();
        let mut obj = obj.clone();
        if obj.key().is_none() {
            obj.set_key(Some(uuid::Uuid::new_v4().to_string()));
        }
        obj.insert(&conn);
        self.emit_change(&obj.type_name(), &obj.key().unwrap());
        obj
    }

    pub fn save<T: LibraryModel>(&self, obj: &T) -> T {
        let conn = self.conn();
        let mut obj = obj.clone();
        if obj.key().is_none() {
            obj.set_key(Some(uuid::Uuid::new_v4().to_string()));
            obj.insert(&conn);
        }
        else {
            obj.update(&conn);
        };
        self.emit_change(&obj.type_name(), &obj.key().unwrap());
        obj
    }

    pub fn get<T: LibraryModel>(&self, key: &str) -> Option<T> {
        let sql = format!("SELECT * FROM {} WHERE key = ?1", T::default().type_name());
        self.conn().query_row(&sql, (key,), 
            |row| Ok(T::from_row(row))).optional().unwrap()
    }

    pub fn list<T: LibraryModel>(&self) -> Vec<T> {
        let sql = format!("SELECT * FROM {}", T::default().type_name());
        self.conn().prepare(&sql).unwrap()
            .query_map((), |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect()
    }

    pub fn query<T: LibraryModel, P: Params>(&self, sql: &str, params: P) -> Vec<T> {
        let conn = self.conn();
        let result = conn.prepare(&sql).unwrap()
            .query_map(params, |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect();
        result
    }

    pub fn find<T: LibraryModel, P: Params>(&self, sql: &str, params: P) -> Option<T> {
        self.conn().query_row(&sql, params, |row| Ok(T::from_row(row))).
            optional().unwrap()
    }

    // Mik's album images are a good test for huge files
    // TODO I think all this fallback stuff actually belongs in the 
    // UI / ImageMangler
    pub fn image<T: LibraryModel>(&self, model: &T) -> Option<DynamicImage> {
        if model.type_name() == "Release" {
            return Release::get(self, &model.key().clone().unwrap()).unwrap()
                .images(self)
                .get(0)
                .and_then(|i| Some(i.get_image()))
        }
        else if model.type_name() == "Artist" {
            let artist = Artist::get(self, &model.key().clone().unwrap()).unwrap();
            if let Some(image) = artist.images(self).get(0) {
                return Some(image.get_image())
            }
            for release in artist.releases(self).iter() {
                if let Some(image) = release.images(self).get(0) {
                    return Some(image.get_image())
                }
            }
        }
        else if model.type_name() == "Track" {
            let track = Track::get(self, &model.key().clone().unwrap()).unwrap();
            if let Some(image) = track.images(self).get(0) {
                return Some(image.get_image())
            }
            if let Some(release) = track.release(self) {
                if let Some(image) = release.images(self).get(0) {
                    return Some(image.get_image())
                }
            }
            for artist in track.artists(self).iter() {
                if let Some(image) = artist.images(self).get(0) {
                    return Some(image.get_image())
                }
            }
        }
        else if model.type_name() == "Genre" {
            let genre = Genre::get(self, &model.key().clone().unwrap()).unwrap();
            if let Some(image) = genre.images(self).get(0) {
                return Some(image.get_image())
            }
            for artist in genre.artists(self).iter() {
                if let Some(image) = artist.images(self).get(0) {
                    return Some(image.get_image())
                }
            }
            for release in genre.releases(self).iter() {
                if let Some(image) = release.images(self).get(0) {
                    return Some(image.get_image())
                }
            }
        }
        None
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
            if let Some(media_file_key) = source.media_file_key {
                if let Some(media_file) = self.get::<MediaFile>(&media_file_key) {
                    if let Ok(content) = std::fs::read(media_file.file_path) {
                        return Some(content)
                    }
                }
            }
        }
        None
    }

    pub fn conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool.get().unwrap()
    }
}

#[derive(Clone)]
pub struct LibraryEvent {
    pub library: Library,
    pub type_name: String,
    pub key: String,
}

impl Debug for LibraryEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryEvent").field("type_name", &self.type_name).field("key", &self.key).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::model::Track;

    use super::Library;

    #[test]
    fn it_works() {
        let _library = Library::open_memory();
    }

    #[test]
    fn load_track_content() {
        let library = Library::open_memory();
        library.import("tests/data/media_files");
        let track = &library.list::<Track>()[0];
        let content = library.load_track_content(track).unwrap();
        assert!(content.len() > 0);
    }

    #[test]
    fn change_notifications() {
        let library = Library::open_memory();
        let (tx, rx) = std::sync::mpsc::channel();
        library.on_change(Box::new(move |_event| {
            tx.send(()).unwrap();
        }));
        library.save(&Track::default());
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
    }
}
