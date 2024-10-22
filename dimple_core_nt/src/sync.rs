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

use crate::library::Library;


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
        println!("Syncing {}", library.uuid());
        // let library_uuid = library.uuid();
        // library.backup(&library_uuid);
        // self.storage.put_file(&library_uuid, &library_uuid);

        let temp_dir = tempdir().unwrap();

        // Download and merge remote libraries.
        println!("Listing remote libraries.");
        let libraries = self.storage.list_objects("dimple.library.");
        libraries.iter().for_each(|library| {
            let contents = self.storage.get_object(library).unwrap();
            let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
            std::fs::write(&temp_file, contents).unwrap();
            println!("Downloaded remote library {} to local file {}.", library, temp_file.to_str().unwrap());
        });

        // Backup the merged input library to a temporary file.
        let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
        library.backup(temp_file.to_str().unwrap());
        println!("Backed up local library to {}.", temp_file.to_str().unwrap());

        // Upload the temporary file to the storage.
        let contents = std::fs::read(temp_file).unwrap();
        let path = format!("dimple.library.{}", library.uuid());
        self.storage.put_object(&path, &contents);
        println!("Uploaded local library to {}.", path);

        // Sync media files
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::{Library, LibraryModel}, model::Track};

    use super::{memory_storage::MemoryStorage, Sync};

    #[test]
    fn it_works() {
        let library1 = Library::open(":memory:");
        Track::default().save(&library1);
        assert!(library1.tracks().len() == 1);

        let library2 = Library::open(":memory:");
        assert!(library2.tracks().len() == 0);

        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage));

        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);

        assert!(library1.tracks().len() == 1);
        assert!(library2.tracks().len() == 1);
    }
}


