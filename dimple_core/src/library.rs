use std::{fmt::Debug, time::Duration};

use dimple_db::{db::{Entity, Migrations}, rusqlite::Params, Db};
use image::DynamicImage;
use include_dir::{include_dir, Dir};
use log::info;
use uuid::Uuid;

use crate::{model::{Artist, Blob, DimpleEntity, Genre, MediaFile, ModelBasics as _, Release, Track, TrackSource}, notifier::Notifier};

#[derive(Clone)]
pub struct Library {
    pub notifier: Notifier<LibraryEvent>,
    db: Db,
}

impl Library {
    pub fn open_memory() -> Self {
        let library = Self {
            db: Db::open_memory().unwrap(),
            notifier: Notifier::new(),
        };

        library.initialize_db();
        library.setup_temporary_notifier();

        library
    }

    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file. If the directory does not exist
    /// it (and all parents) will be created.
    pub fn open(database_path: &str) -> Self {
        let library = Self {
            db: Db::open(database_path).unwrap(),
            notifier: Notifier::new(),
        };

        library.initialize_db();
        library.setup_temporary_notifier();

        library
    }

    fn setup_temporary_notifier(&self) {
        let library_clone = self.clone();
        std::thread::spawn(move || {
            let rx = library_clone.db.subscribe();
            while let Ok(event) = rx.recv() {
                match event {
                    dimple_db::db::DbEvent::Insert(type_name, id) => library_clone.notifier.notify(LibraryEvent { 
                        library: library_clone.clone(), 
                        type_name: type_name, 
                        key: id, 
                    }),
                    dimple_db::db::DbEvent::Update(type_name, id) => library_clone.notifier.notify(LibraryEvent { 
                        library: library_clone.clone(), 
                        type_name: type_name, 
                        key: id, 
                    }),
                    dimple_db::db::DbEvent::Delete(type_name, id) => library_clone.notifier.notify(LibraryEvent { 
                        library: library_clone.clone(), 
                        type_name: type_name, 
                        key: id, 
                    }),
                }
            }
        });
    }

    fn initialize_db(&self) {
        static MIGRATION_DIR: Dir = include_dir!("./src/migrations");
        let migrations = Migrations::from_directory(&MIGRATION_DIR).unwrap();
        self.db.migrate(&migrations).unwrap()
    }

    /// Returns the unique, permanent ID of this Library. This is created when
    /// the Library is created and doesn't change.
    pub fn id(&self) -> String {
        self.db.get_database_uuid().unwrap()
    }

    /// Backup this library to the specified path.
    pub fn backup(&self, output_path: &str) {
        // TODO pass through to db.
        todo!()
        // let mut dst = Connection::open(output_path).unwrap();
        // let src = self.conn();
        // let backup = Backup::new(&src, &mut dst).unwrap();
        // // TODO maybe return a stream of events for progress or something
        // // TODO magic
        // backup.run_to_completion(250, Duration::from_millis(10), None).unwrap();
        // self.db.get_database_uuid().unwrap()
    }

    /// Import MediaFiles into the Library, creating or updating Tracks,
    /// TrackSources, Blobs, etc. path can be either a file or directory. If
    /// it is a directory it will be recursively scanned.
    /// TODO this goes away and into plugins too, I think.
    pub fn import(&self, path: &str) {
        crate::import::import(self, path);
     }

    pub fn sync(&self) {
        todo!()
        // if let Ok(syncs) = self.synchronizers.read() {
        //     for sync in syncs.iter() {
        //         sync.sync(self);
        //     }
        // }
    }

    // TODO need to change these wrappers to return Results
    pub fn save<T: Entity>(&self, obj: &T) -> T {
        self.db.save(obj).unwrap()
    }

    pub fn get<T: Entity>(&self, key: &str) -> Option<T> {
        self.db.get(key).unwrap()
    }

    pub fn list<T: Entity>(&self) -> Vec<T> {
        let sql = format!("SELECT * FROM {}", self.db.table_name_for_type::<T>().unwrap());
        self.db.query(&sql, ()).unwrap()
    }

    pub fn query<T: Entity, P: Params>(&self, sql: &str, params: P) -> Vec<T> {
        self.db.query(sql, params).unwrap()
    }

    pub fn find<T: Entity, P: Params>(&self, sql: &str, params: P) -> Option<T> {
        self.query(sql, params).into_iter().next()
    }

    // Mik's album images are a good test for huge files
    // TODO I think all this fallback stuff actually belongs in the 
    // UI / ImageMangler
    pub fn image(&self, model: &DimpleEntity) -> Option<DynamicImage> {
        match model {
            DimpleEntity::Artist(artist) => {
                if let Some(image) = artist.images(self).get(0) {
                    return Some(image.get_image())
                }
                for release in artist.releases(self).iter() {
                    if let Some(image) = release.images(self).get(0) {
                        return Some(image.get_image())
                    }
                }
            },
            DimpleEntity::Track(track) => {
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
            },
            DimpleEntity::Release(release) => {
                return release.images(self)
                    .get(0)
                    .and_then(|i| Some(i.get_image()))
            },
            DimpleEntity::Genre(genre) => {
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
            _ => ()
        }
        None
    }

    pub fn find_media_file_by_file_path(&self, file_path: &str) -> Option<MediaFile> {
        self.find("SELECT * FROM MediaFile WHERE file_path = ?", (file_path,))
    }

    pub fn find_blob_by_sha256(&self, sha256: &str) -> Option<Blob> {
        self.find("SELECT * FROM Blob WHERE sha256 = ?", (sha256,))
    }

    pub fn track_sources_for_track(&self, track: &Track) -> Vec<TrackSource> {
        self.query("SELECT * FROM TrackSource WHERE track_id = ?", (&track.id,))
    }
        
    pub fn track_sources_by_blob(&self, blob: &Blob) -> Vec<TrackSource> {
        self.query("SELECT * FROM TrackSource WHERE blob_id = ?", (&blob.id,))
    }

    pub fn media_files_by_sha256(&self, sha256: &str) -> Vec<MediaFile> {
        self.query("SELECT * FROM TrackSource WHERE sha256 = ?", (sha256,))
    }

    pub fn load_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
        for media_file in self.media_files_by_sha256(&blob.sha256) {
            if let Ok(content) = std::fs::read(&media_file.file_path) {
                info!("Found blob sha256 {} at {}", blob.sha256, &media_file.file_path);
                return Some(content)
            }
        }
        // TODO This will go to Db
        // for sync in self.synchronizers.read().unwrap().iter() {
        //     if let Some(content) = sync.load_blob_content(blob) {
        //         info!("Found blob sha256 {} in sync", blob.sha256);
        //         return Some(content)
        //     }
        // }
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
            if let Some(blob_id) = source.blob_id {
                if let Some(blob) = self.get::<Blob>(&blob_id) {
                    if let Some(content) = self.load_blob_content(&blob) {
                        return Some(content)
                    }
                }
            }
            if let Some(media_file_id) = source.media_file_id {
                if let Some(media_file) = self.get::<MediaFile>(&media_file_id) {
                    if let Ok(content) = std::fs::read(media_file.file_path) {
                        return Some(content)
                    }
                }
            }
        }
        None
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
        library.notifier.observe(move |_event| {
            tx.send(()).unwrap();
        });
        library.save(&Track::default());
        assert!(rx.recv_timeout(Duration::from_millis(100)).is_ok());
    }
}
