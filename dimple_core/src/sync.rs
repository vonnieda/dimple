/// Sync a Library with a compatible storage target. Allows multiple
/// devices to share the same library. Designed for S3 but adaptable with
/// the Storage trait.
/// 
/// The sync protocol is designed as a multi-writer distributed operation log.
/// Each change to the database is logged in the ChangeLog table as one or more
/// operations, along with a ulid logical timestamp and an actor id based on
/// the library uuid. 
/// 
/// Each actor / client maintains their own copy of the database, including
/// the ChangeLog. When a library we merge any remote ChangeLogs that are found,
/// apply any that are newer than those previously observed, and then push the
/// combined ChangeLog up to the remote. As each actor performs these actions
/// the individual databases converge to the same values.
/// 
/// When creating an Sync instance you specify a path on the Storage to use
/// as a base. This is prepended along with a / for all storage operations.
/// 
/// By using a guaranteed unique Sync path like a UUID we can store multiple
/// libraries on the same storage, or we can store shares.

pub mod storage;
pub mod s3_storage;
pub mod memory_storage;

use std::collections::HashSet;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use log::{info, warn};
use storage::Storage;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{library::Library, model::{Blob, ChangeLog, Diff, Model, Track, TrackSource}};

pub struct Sync {
    storage: Box<dyn Storage>,
    path: String,
}

impl Sync {
    pub fn new(storage: Box<dyn Storage>, path: &str) -> Self {
        Sync {
            storage,
            path: path.to_string(),
        }
    }

    /// TODO library will need to maintain a reference to sync for looking up
    ///      blobs when it's online.
    /// 
    /// # Goals
    /// 
    /// - Sync multiple devices via one S3 prefix.
    /// - Create share URLs that allow anyone with the URL to add / listen in
    ///   Dimple. This involves creating pre-signed URLs for partial databases
    ///   and the associated blobs.
    /// 
    /// 
    /// # File Layout
    /// 
    /// - {path}/db/{library.id()}.db      
    ///   Databases of devices participating in the sync.
    /// 
    /// - {path}/blobs/{blob.sha256}.blob
    ///   Blobs stored under their SHA256 for de-dupe. This includes media,
    ///   images, cover art, etc.
    /// 
    /// - {path}/shares/{share.id()}.db    
    ///   Database of shared info for a specific share. Shared via pre-signed URL
    ///   and includes pre-signed URLs to reference the blobs.
    /// 
    /// I think this is actually going to reflect the layout on local disk too.
    /// 
    pub fn sync(&self, library: &Library) {
        info!("Synchronizing {}.", library.id());
        let temp_dir = tempdir().unwrap();

        {
            info!("Pulling remote changes.");
            let remote_library_paths = self.storage.list_objects(&format!("{}/db/", self.path));
            info!("Remote libraries {:?}", remote_library_paths);
            remote_library_paths.iter().for_each(|remote_library_path| {
                let contents = self.storage.get_object(remote_library_path).unwrap();
                let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
                info!("Downloading {} to {}.", remote_library_path, temp_file.to_str().unwrap());
                std::fs::write(&temp_file, &contents).unwrap();
    
                info!("Opening library {}.", temp_file.to_str().unwrap());
                let remote_library = Library::open(temp_file.to_str().unwrap());
    
                if remote_library.id() == library.id() {
                    info!("Skipping own library with same id.");
                    return
                }
    
                info!("Library contains {} tracks and {} changelogs.",
                    remote_library.tracks().len(),
                    remote_library.changelogs().len());
    
                let changelogs = remote_library.changelogs();
                info!("Applying {} changelogs", changelogs.len());
    
                for changelog in changelogs {
                    Self::apply_changelog(library, &changelog);
                    // TODO this is redundant if the change has already been seen
                    // and causes lots of slow.
                    library.save(&changelog);
                }
            });
        }

        {
            // Upload a copy of the input library to storage.
            // TODO Eventually just upload the changelog, and then further just
            // what has changed since last time.
            let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
            info!("Gathering local changes.");
            library.backup(temp_file.to_str().unwrap());
            let path = format!("{}/db/{}.db", self.path, library.id());
            let contents = std::fs::read(temp_file).unwrap();
            info!("Pushing local changes.");
            self.storage.put_object(&path, &contents);
        }

        {
            // Sync blobs
            info!("Syncing blobs");
            let local_blobs: Vec<Blob> = library.list::<Blob>();
            let remote_blob_names: HashSet<String> = self.storage
                .list_objects(&format!("{}/blobs/", self.path))
                .iter()
                .map(|n| n.rsplit_once("/").unwrap().1.to_string())
                .collect();
            let to_store: Vec<Blob> = local_blobs.into_iter()
                .filter(|b| !remote_blob_names.contains(&format!("{}.blob", b.sha256)))
                .collect();
            info!("Pushing {} new blobs.", to_store.len());
            to_store.par_iter().for_each(|blob| {
                if let Some(content) = library.load_local_blob_content(&blob) {
                    let path = format!("{}/blobs/{}.blob", self.path, blob.sha256);
                    info!("Pushing blob {}.", path);
                    self.storage.put_object(&path, &content);
                }
                else {
                    warn!("No content found to sync for sha256 {}", blob.sha256);
                }
            });
        }

        // TODO also pull down new blobs that are marked for offline.

        info!("Sync complete.");
    }

    pub fn load_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
        let path = format!("{}/blobs/{}.blob", self.path, blob.sha256);
        self.storage.get_object(&path)
    }

    fn apply_changelog(library: &Library, changelog: &ChangeLog) {
        let actor = changelog.actor.clone();
        let timestamp = changelog.timestamp.clone();
        let model = changelog.model.clone();
        let model_key = changelog.model_key.clone();
        let op = changelog.op.clone();
        // Ignore our own changes. Could be disabled to rebuild the db.
        if actor == library.id() {
            return
        }
        // TODO generify
        if model == "Track" {
            // TODO duplicated check of set in apply_diff
            if op == "set" {
                let field = changelog.field.clone().unwrap();
                if let Some(newest_changelog) = library.find_newest_changelog_by_field(&model, &model_key, &field) {
                    if newest_changelog.timestamp >= timestamp {
                        return
                    }
                }
                let mut obj = library.get(&model_key)
                    .or_else(|| Some(Track { key: Some(model_key.clone()), ..Default::default() })).unwrap();
                obj.apply_diff(&[changelog.clone()]);
                library.save_unlogged(&obj);
            }
        }
        if model == "TrackSource" {
            // TODO duplicated check of set in apply_diff
            if op == "set" {
                let field = changelog.field.clone().unwrap();
                if let Some(newest_changelog) = library.find_newest_changelog_by_field(&model, &model_key, &field) {
                    if newest_changelog.timestamp >= timestamp {
                        return
                    }
                }
                let mut obj = library.get(&model_key)
                    .or_else(|| Some(TrackSource { key: Some(model_key.clone()), ..Default::default() })).unwrap();
                obj.apply_diff(&[changelog.clone()]);
                library.save_unlogged(&obj);
            }
        }
        if model == "Blob" {
            // TODO duplicated check of set in apply_diff
            if op == "set" {
                let field = changelog.field.clone().unwrap();
                if let Some(newest_changelog) = library.find_newest_changelog_by_field(&model, &model_key, &field) {
                    if newest_changelog.timestamp >= timestamp {
                        return
                    }
                }
                let mut obj = library.get(&model_key)
                    .or_else(|| Some(Blob { key: Some(model_key.clone()), ..Default::default() })).unwrap();
                obj.apply_diff(&[changelog.clone()]);
                library.save_unlogged(&obj);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::Track};

    use super::{memory_storage::MemoryStorage, Sync};

    #[test]
    fn it_works() {
        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage), "TODO");

        let library1 = Library::open("file:5cf6bba9-f63b-4090-a944-114d224e25b7?mode=memory&cache=shared");
        library1.save(&Track { 
            artist: Some("Grey Speaker".to_string()), 
            title: Some("One Thing".to_string()), 
            ..Default::default() 
        });
        sync.sync(&library1);

        let library2 = Library::open("file:4e3db7d3-042a-4770-a1c5-2c53289cad46?mode=memory&cache=shared");
        library2.save(&Track { 
            title: Some("Tall Glass".to_string()), 
            ..Default::default() 
        });
        sync.sync(&library2);

        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);

        assert!(library1.tracks().len() == 2);
        assert!(library2.tracks().len() == 2);
        assert!(library1.changelogs().len() == 5);
        assert!(library2.changelogs().len() == 5);
        assert!(library1.changelogs() == library2.changelogs());
    }

    #[test]
    fn big_library() {
        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage), "e447a237-4930-468d-a471-50bd775080a4");

        let library = Library::open("file:c041451b-86d0-43c6-974d-84d5866691e3?mode=memory&cache=shared");
        for i in 0..300 {
            library.save(&Track { 
                artist: Some(format!("Grey Speaker {}", i)), 
                title: Some(format!("One Thing {}", i)), 
                ..Default::default() 
            });
        }
        sync.sync(&library);

        let library2 = Library::open("file:9511639e-7fdd-4fa1-9d72-c458f1696114?mode=memory&cache=shared");
        sync.sync(&library2);
    }
}


