use std::fmt::Debug;

use anyhow::Result;
use dimple_db::{db::{Entity, Migrations}, rusqlite::Params, Db};
use image::DynamicImage;
use include_dir::{include_dir, Dir};

use crate::{model::{DimpleEntity, MediaFile, ModelBasics as _, Track, TrackSource}, notifier::Notifier};

#[derive(Clone)]
pub struct Library {
    pub db: Db,
}

impl Library {
    pub fn open_memory() -> Self {
        let library = Self {
            db: Db::open_memory().unwrap(),
        };

        library.initialize_db();

        library
    }

    /// Open the library located at the specified path. The path is to an
    /// optionally existing Sqlite database. Blobs will be stored in the
    /// same directory as the specified file. If the directory does not exist
    /// it (and all parents) will be created.
    pub fn open(database_path: &str) -> Self {
        let library = Self {
            db: Db::open(database_path).unwrap(),
        };

        library.initialize_db();

        library
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
    pub fn save<T: Entity>(&self, obj: &T) -> Result<T> {
        self.db.save(obj)
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
                if let Some(image) = artist.images(self).first() {
                    return Some(image.get_image())
                }
                for release in artist.releases(self).iter() {
                    if let Some(image) = release.images(self).first() {
                        return Some(image.get_image())
                    }
                }
            },
            DimpleEntity::Track(track) => {
                if let Some(image) = track.images(self).first() {
                    return Some(image.get_image())
                }
                if let Some(release) = track.release(self) {
                    if let Some(image) = release.images(self).first() {
                        return Some(image.get_image())
                    }
                }
                for artist in track.artists(self).iter() {
                    if let Some(image) = artist.images(self).first() {
                        return Some(image.get_image())
                    }
                }
            },
            DimpleEntity::Release(release) => {
                return release.images(self).first().map(|i| i.get_image())
            },
            DimpleEntity::ReleaseGroup(release_group) => {
                return release_group.images(self).first().map(|i| i.get_image())
            },
            DimpleEntity::Genre(genre) => {
                if let Some(image) = genre.images(self).first() {
                    return Some(image.get_image())
                }
                for artist in genre.artists(self).iter() {
                    if let Some(image) = artist.images(self).first() {
                        return Some(image.get_image())
                    }
                }
                for release in genre.releases(self).iter() {
                    if let Some(image) = release.images(self).first() {
                        return Some(image.get_image())
                    }
                }
            }
            _ => ()
        }
        None
    }

    pub fn track_sources_for_track(&self, track: &Track) -> Vec<TrackSource> {
        self.query("SELECT * FROM TrackSource WHERE track_id = ?", (&track.id,))
    }
        
    /// TODO so, to do this right I'm eventually going to have to support external
    /// explicit blobs in dimple_db. In the general case I need to be able to
    /// de-dupe them in the changelog, and also not store them in the
    /// changelog and the database AND the original file
    /// So two things:
    /// 1. We'll only have one blob in the entire database, in the Blob model.
    /// 2. In dimple_db, my preference would be to store the blob column in
    /// the database so that queries remain normal. Which means we don't store
    /// it in the changelog. So sync would need to be aware. 
    /// Another option, maybe a lot better:
    /// dimple_db always treats blob columns as file references. Sync is
    /// responsible for syncing the files. 
    /// Maybe this is just a special column marker or something actually.
    /// So we have a content_id -> path mapping stored in the database. 
    /// 
    /// Create a Blob record in the database from the bytes. Stores the blob
    /// content in the database. Returns the Blob.
    /// Db.blob_from_bytes(bytes) -> Result<Blob>;
    /// 
    /// Create a Blob record in the database from the content of the given
    /// file. Returns the Blob.
    /// Db.blob_from_file(path) -> Result<Blob>;
    /// 
    /// Look up a Blob by it's sha256 hash. 
    /// Db.blob(sha256) -> Option<Blob>;
    /// 
    /// Read a Blob's content.
    /// let bytes = Blob.read(&self);
    /// 
    /// These are really the only operations I need. On import I can create
    /// blobs from media files, and everything else gets handled internally.
    pub fn load_track_content(&self, track: &Track) -> Option<Vec<u8>> {
        for source in self.track_sources_for_track(track) {
            if let Some(media_file_id) = source.media_file_id {
                if let Some(media_file) = self.get::<MediaFile>(&media_file_id) {
                    if let Some(content) = media_file.content {
                        return Some(content)
                    }
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
}
