use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}, time::Duration};

use image::DynamicImage;
use include_dir::{include_dir, Dir};
use log::info;
use rusqlite::{backup::Backup, Connection, OptionalExtension, Params};
use rusqlite_migration::Migrations;
use threadpool::ThreadPool;
use ulid::Generator;
use uuid::Uuid;

use crate::{model::{Blob, ChangeLog, FromRow, MediaFile, Model, Playlist, Track, TrackSource}, notifier::Notifier, sync::Sync};

#[derive(Clone)]
pub struct LibraryEvent {
    pub library: Library,
    pub type_name: String,
    pub key: String,
}

#[derive(Clone)]
pub struct Library {
    conn: Arc<Mutex<Connection>>,
    database_path: String,
    ulids: Arc<Mutex<Generator>>,
    // TODO I really think I want to get rid of this and put it somewhere
    // higher level. Note: Yea, it's going into Plugins and we're deleting
    // sync from Library entirely.
    synchronizers: Arc<RwLock<Vec<Sync>>>,
    notifier: Notifier<LibraryEvent>,
    threadpool: ThreadPool,
}

impl Library {
    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file. If the directory does not exist
    /// it (and all parents) will be created.
    pub fn open(database_path: &str) -> Self {
        // std::fs::create_dir_all(path).unwrap();
        // let db_path = Path::new(path).join("library.db");
        // let plugin_cache_path = Path::new(path).join("plugin_cache");
        // fs::create_dir_all(plugin_cache_path.clone()).unwrap();
        // let librarian = Self {
        //     db: SqliteDb::new(db_path.to_str().unwrap()).unwrap(),
        //     plugins: Default::default(),
        //     network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
        //     plugin_context: PluginContext::new(plugin_cache_path.to_str().unwrap()),
        // };
        // librarian

        static MIGRATION_DIR: Dir = include_dir!("./dimple_core/src/migrations");
        let migrations = Migrations::from_directory(&MIGRATION_DIR).unwrap();

        let mut conn = Connection::open(database_path).unwrap();

        migrations.to_latest(&mut conn).unwrap();

        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();        

        conn.execute("
            INSERT INTO Metadata (key, value) VALUES ('library.uuid', ?1)
            ON CONFLICT DO NOTHING
            ",
            (Uuid::new_v4().to_string(),),
        ).unwrap();

        let library = Library {
            conn: Arc::new(Mutex::new(conn)),
            database_path: database_path.to_string(),
            ulids: Arc::new(Mutex::new(Generator::new())),
            synchronizers: Arc::new(RwLock::new(vec![])),
            notifier: Notifier::new(),
            threadpool: ThreadPool::new(1),
        };

        library
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
        if obj.log_changes() {
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
        self.emit_change(&obj.type_name(), &obj.key().unwrap());
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
        let sql = format!("SELECT * FROM {} WHERE key = ?1", T::default().type_name());
        self.conn().query_row(&sql, (key,), 
            |row| Ok(T::from_row(row))).optional().unwrap()
    }

    pub fn list<T: Model>(&self) -> Vec<T> {
        let sql = format!("SELECT * FROM {}", T::default().type_name());
        self.conn().prepare(&sql).unwrap()
            .query_map((), |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect()
    }

    pub fn query<T: Model, P: Params>(&self, sql: &str, params: P) -> Vec<T> {
        let conn = self.conn();
        let result = conn.prepare(&sql).unwrap()
            .query_map(params, |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect();
        result
    }

    // TODO Need to profile the whole image path and see
    // why it's so slow in dev mode. It's just unbearable. And maybe I really
    // just do need to use smaller images? Mik's album images are a good test.
    pub fn image<T: Model>(&self, model: &T) -> Option<DynamicImage> {
        // let t = Instant::now();
        // let dimage = self.db.list(&Dimage::default().into(), &Some(model.clone()))
        //     .unwrap()
        //     .map(Into::<Dimage>::into)
        //     .next();
        // if let Some(dimage) = dimage {
        //     log::debug!("image from database {:?} {}x{} in {}ms", dimage.key, dimage.width, 
        //         dimage.height, t.elapsed().as_millis());
        //     return Some(dimage.get_image())
        // }

        // // TODO note, this uses a specialization of list that returns on the 
        // // first valid result to speed things up. Eventually I want Dimage to
        // // not include the blob, and then this won't be needed, or wanted,
        // // because we'll want to be able to offer the user all the different
        // // images, not just one.
        // let t = Instant::now();
        // let dimage = self._list(&Dimage::default().into(), &Some(model.clone()), true)
        //     .unwrap()
        //     .map(Into::<Dimage>::into)
        //     .next();
        // if let Some(dimage) = dimage {
        //     log::debug!("image from plugins {:?} {}x{} in {}ms", dimage.key, dimage.width, 
        //         dimage.height, t.elapsed().as_millis());
        //     return Some(dimage.get_image())
        // }

        // // If nothing found specific to the model, see if there's something related.
        // let t = Instant::now();
        // match model {
        //     Model::Artist(artist) => {
        //         let release_groups = self.list2(ReleaseGroup::default(), Some(artist.clone()));
        //         if let Ok(release_groups) = release_groups {
        //             for release_group in release_groups {
        //                 if let Some(dimage) = self.image(&release_group.model()) {
        //                     log::debug!("image from relations {}x{} in {}ms", dimage.width(), 
        //                         dimage.height(), t.elapsed().as_millis());
        //                     return Some(dimage)
        //                 }
        //             }
        //         }
        //     },
        //     Model::Genre(genre) => {
        //         let release_groups = self.list2(ReleaseGroup::default(), Some(genre.clone()));
        //         if let Ok(release_groups) = release_groups {
        //             for release_group in release_groups {
        //                 if let Some(dimage) = self.image(&release_group.model()) {
        //                     log::debug!("image from relations {}x{} in {}ms", dimage.width(), 
        //                         dimage.height(), t.elapsed().as_millis());
        //                     return Some(dimage)
        //                 }
        //             }
        //         }
        //         let artists = self.list2(Artist::default(), Some(genre.clone()));
        //         if let Ok(artists) = artists {
        //             for artist in artists {
        //                 if let Some(dimage) = self.image(&artist.model()) {
        //                     log::debug!("image from relations {}x{} in {}ms", dimage.width(), 
        //                         dimage.height(), t.elapsed().as_millis());
        //                     return Some(dimage)
        //                 }
        //             }
        //         }
        //     }
        //     _ => ()
        // }

        None
    }

    // TODO now that library.query takes params most of these can be moved into
    // their caller's code
    pub fn changelogs(&self) -> Vec<ChangeLog> {
        self.query("SELECT * FROM ChangeLog ORDER BY timestamp ASC", ())
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.query("SELECT * FROM Track ORDER BY artist, album, title", ())
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
            if let Some(blob) = self.get::<Blob>(&source.blob_key) {
                if let Some(content) = self.load_blob_content(&blob) {
                    return Some(content)
                }
            }
        }
        None
    }

    fn conn(&self) -> Connection {
        // TODO wait, wait, why did I change this from a single connection?
        // Cause that is probably slowing things down a lot. So if I need
        // it this way (why?) then I need to pool.
        // > library conn() is now a function that returns a new connection - 
        // > needed to make Library sharable, and this paves the way for
        // > transactions. Previously would have needed mut Library.
        Connection::open(self.database_path.clone()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::model::{Diff, MediaFile, Model, Track};

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
        library.import("tests/data/media_files");
        let num_mediafiles = library.list::<MediaFile>().len();
        assert!(library.list::<MediaFile>().len() > 0);
        library.import("tests/data/media_files");
        assert!(library.list::<MediaFile>().len() == num_mediafiles);
    }

    #[test]
    fn load_track_content() {
        let library = Library::open("file:6384d9e0-74c1-4e1d-9ea3-b5d0198f134e?mode=memory&cache=shared");
        library.import("tests/data/media_files");
        let track = &library.tracks()[0];
        let content = library.load_track_content(track).unwrap();
        assert!(content.len() > 0);
    }

    #[test]
    fn change_notifications() {
        let library = Library::open("file:40c65129-2e84-4eff-8b25-9a03519da1e1?mode=memory&cache=shared");
        // let (tx, rx) = std::sync::mpsc::channel();
        // library.on_change(move |_library, type_name, _key| if type_name == "Track" {
        //     tx.send(()).unwrap();
        // });
        library.on_change(Box::new(move |event| {
            println!("{} {}", event.type_name, event.key);
        }));
        library.save(&Track::default());
        // assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
    }
}
