/// Sync a Library with an S3 compatible storage target. Allows multiple
/// devices to share the same library.
/// 
/// Hybrid Logical Clocks + Op Log
/// 
/// What if Sync listened to changes to the database and created the op log
/// on it's own, without support from the model. I guess this is a WAL, sort of.
/// 
/// So it observes Library for changes, and writes those changes to S3. It also
/// reads changes from other peers, merges all the changes together, and then
/// calculates if any local values need to be changed based on the HLC (Hybrid
/// Logical Clock)
/// 
/// The if the WAL gets lost it doesn't matter because the DB itself is up to
/// date and can be transferred in whole. The purpose of the WAL is to upload
/// only changes instead of the whole database, most of the time.
/// 
/// So, the way merge works:
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
/// 
/// What happens if someone's op log gets corrupt? Or deleted? Or they rebuild
/// their library from scratch?
///     Choose to adopt a backup?
/// 
/// 
/// 

pub mod storage;
pub mod s3_storage;
pub mod memory_storage;

use std::io::Write;

use storage::Storage;
use tempfile::{tempdir, tempfile};
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
        // let library_uuid = library.uuid();
        // library.backup(&library_uuid);
        // self.storage.put_file(&library_uuid, &library_uuid);

        let temp_dir = tempdir().unwrap();

        // Download and merge remote libraries.
        let libraries = self.storage.list_objects("dimple.library.");
        libraries.iter().for_each(|library| {
            let contents = self.storage.get_object(library).unwrap();
            let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
            std::fs::write(&temp_file, contents).unwrap();
            println!("Downloaded {} to {}", library, temp_file.to_str().unwrap());
        });

        // Backup the merged input library to a temporary file.

        // Upload the temporary file to the storage.

        // Download and upload media files
    }
}

#[cfg(test)]
mod tests {
    use crate::library::Library;

    use super::{memory_storage::MemoryStorage, s3_storage::S3Storage, Sync};

    #[test]
    fn basics() {
        let library1 = Library::open(":memory:");

        let library2 = Library::open(":memory:");

        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage));

        sync.sync(&library1);
        sync.sync(&library2);
    }
}


// let s3_path = "test.file";
// let test = b"I'm going to S3!";

// let response_data = bucket.put_object(s3_path, test)?;
// assert_eq!(response_data.status_code(), 200);

// let response_data = bucket.get_object(s3_path)?;
// assert_eq!(response_data.status_code(), 200);
// assert_eq!(test, response_data.as_slice());

// let response_data = bucket.get_object_range(s3_path, 100, Some(1000))?;
// assert_eq!(response_data.status_code(), 206);
// let (head_object_result, code) = bucket.head_object(s3_path)?;
// assert_eq!(code, 200);
// assert_eq!(
//     head_object_result.content_type.unwrap_or_default(),
//     "application/octet-stream".to_owned()
// );

// let response_data = bucket.delete_object(s3_path)?;
// assert_eq!(response_data.status_code(), 204);
