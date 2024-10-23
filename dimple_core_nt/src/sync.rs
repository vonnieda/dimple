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
        // TODO need to decide here if it should be applied based on the
        // HLC. And now I'm facing the trigger problem finally. Each write
        // triggers a changelog.
        // So, can I disable the triggers? Maybe in a txn? Or do I need to
        // switch away from the triggers?
        // Can't disable, but could probably hack something together.
        // What does switching away look like?
        // I guess the save function would need to call save_changelog() and
        // could then also send events to observers. This also is going to
        // make it easier to use proper str or int values.
        // At that point, maybe just store json?
        // Easy enough to convert later.

        let changelog = changelog.clone();
        let actor = changelog.actor;
        let timestamp = changelog.timestamp;
        let model = changelog.model;
        let key = changelog.key;
        let op = changelog.op;
        if model == "Track" {
            // TODO only partially complete. Not sure if this is how it
            // works eventually (can't I just emit SQL?) but it does
            // work.
            if op == "insert_field" || op == "set_field" {
                let field = changelog.field.unwrap();
                let mut track = Track::get(library, &key)
                    .or_else(|| Some(Track { key: Some(key), ..Default::default() }.save(library))).unwrap();
                if field == "artist" {
                    track.artist = changelog.value;
                    track.save(library);
                }
                else if field == "album" {
                    track.album = changelog.value;
                    track.save(library);
                }
                else if field == "title" {
                    track.title = changelog.value;
                    track.save(library);
                }
                else if field == "path" {
                    track.path = changelog.value.unwrap();
                    track.save(library);
                }
                else if field == "liked" {
                    track.liked = changelog.value.is_some() && changelog.value.unwrap() == "1";
                    track.save(library);
                }
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
        let library1 = Library::open(":memory:");
        Track::default().save(&library1);
        Track::default().save(&library1);
        Track::default().save(&library1);
        Track::default().save(&library1);
        let mut track = Track::default().save(&library1);
        track.liked = true;
        track.save(&library1);
        track.liked = false;
        track.save(&library1);
        track.liked = false;
        track.save(&library1);
        track.liked = true;
        track.save(&library1);
        // Track { artist: Some("Ten Hooligans".to_string()), ..Default::default() }.save(&library1);
        // Track { key: Some("1234".to_string()), artist: Some("Doops Dog".to_string()), ..Default::default() }.save(&library1);
        // assert!(library1.tracks().len() == 2);

        let library2 = Library::open(":memory:");
        Track::default().save(&library2);
        // assert!(library2.tracks().len() == 1);

        let library3 = Library::open(":memory:");
        // Track { key: Some("1234".to_string()), artist: Some("Swaggy".to_string()), ..Default::default() }.save(&library1);
        // assert!(library3.tracks().len() == 0);

        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage));

        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library3);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);

        // TODO this is all looking great. need to work on the clock and the
        // rest of the fields.
        // Maybe model.create_changelog() and model.apply_changelog()?

        // assert!(library1.tracks().len() == 3);
        // assert!(library2.tracks().len() == 3);
        // assert!(library3.tracks().len() == 3);
    }
}


