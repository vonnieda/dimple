/// Sync a Library with an S3 compatible storage target. Allows multiple
/// devices to share the same library.
/// 
/// Hybrid Logical Clock + Op Log
/// 
/// We have an HLC (Hybrid Logical Clock) per Library which is used to order 
/// changes between replicas. 
/// 
/// Every time a change is made to the library we both store the change and we
/// store a new row in the op log. The op log contains the next HLC output,
/// the model type, the property, the old value, and the new value. 
/// 
/// We merge them all together, sort by HLC, and then take the values for
/// each property as the max of the HLC.

pub mod storage;
pub mod s3_storage;
pub mod memory_storage;


use storage::Storage;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{library::{Library, LibraryModel}, model::{ChangeLog, Track}};


pub struct Sync {
    storage: Box<dyn Storage>,
}

impl Sync {
    pub fn new(storage: Box<dyn Storage>) -> Self {
        Sync {
            storage,
        }
    }

    pub fn sync(&self, library: &Library) {
        println!("Syncing library {}.", library.uuid());

        let temp_dir = tempdir().unwrap();

        // // Create a temporary database that will be used to merge changes.
        // let working_db_path = temp_dir.path().join(Uuid::new_v4().to_string());
        // let working_library = Library::open(working_db_path.to_str().unwrap());

        // Download and merge remote libraries.
        println!("  Listing remote libraries.");
        let remote_library_paths = self.storage.list_objects("dimple.library.");
        remote_library_paths.iter().for_each(|remote_library_path| {
            println!("    Getting remote library {}.", remote_library_path);
            let contents = self.storage.get_object(remote_library_path).unwrap();
            let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
            std::fs::write(&temp_file, contents).unwrap();

            println!("      Opening remote library.");
            let remote_library = Library::open(temp_file.to_str().unwrap());

            println!("      Loading change logs.");
            let changelogs = remote_library.changelogs();

            println!("      Merging {} change logs.", changelogs.len());
            for changelog in changelogs {
                changelog.save(library);
                Self::apply_changelog(library, &changelog);
            }
        });

        // Upload a backup of the input library to storage.
        let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
        library.backup(temp_file.to_str().unwrap());
        let path = format!("dimple.library.{}", library.uuid());
        println!("  Uploading local library to {}.", path);
        let contents = std::fs::read(temp_file).unwrap();
        self.storage.put_object(&path, &contents);

        // Sync media files
    }

    fn apply_changelog(library: &Library, changelog: &ChangeLog) {
        let actor = changelog.actor.clone();
        let timestamp = changelog.timestamp.clone();
        let model = changelog.model.clone();
        let key = changelog.key.clone();
        let op = changelog.op.clone();
        if actor == library.uuid() {
            return
        }
        if model == "Track" {
            // TODO duplicated check of set in Track::apply_diff
            if op == "set" {
                let mut track = Track::get(library, &key)
                    .or_else(|| Some(Track { key: Some(key), ..Default::default() })).unwrap();
                track.apply_diff(&[changelog.clone()]);
                track.save(library);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::{Library, LibraryModel}, model::Track};

    use super::{memory_storage::MemoryStorage, Sync};

    #[test]
    fn it_works() {
        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage));

        let library1 = Library::open(":memory:");
        Track { title: Some("One Thing".to_string()), ..Default::default() }.save(&library1);
        sync.sync(&library1);

        let library2 = Library::open(":memory:");
        Track { title: Some("Tall Glass".to_string()), ..Default::default() }.save(&library2);
        sync.sync(&library2);

        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);

        /// Okay, so this is all sort of working except we're duplicating
        /// the changelogs because we're not checking if we already merged
        /// it, and we're creating changelogs when merging changes.

        assert!(library1.tracks().len() == 2);
        assert!(library2.tracks().len() == 2);
        dbg!(library1.tracks());
        dbg!(library2.tracks());
        dbg!(library1.changelogs());
    }
}


