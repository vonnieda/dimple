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

use storage::Storage;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{library::Library, model::{ChangeLog, Diff, Track}};

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

    pub fn sync(&self, library: &Library) {
        println!("Synchronizing {}", library.id());
        let temp_dir = tempdir().unwrap();
        let remote_library_paths = self.storage.list_objects(&format!("{}/", self.path));
        println!("Remote libraries {:?}", remote_library_paths);
        remote_library_paths.iter().for_each(|remote_library_path| {
            let contents = self.storage.get_object(remote_library_path).unwrap();
            let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
            println!("Downloading {} to {}", remote_library_path, temp_file.to_str().unwrap());
            std::fs::write(&temp_file, &contents).unwrap();

            println!("Opening {}", temp_file.to_str().unwrap());
            let remote_library = Library::open(temp_file.to_str().unwrap());

            println!("Library contains {} tracks and {} changelogs.",
                remote_library.tracks().len(),
                remote_library.changelogs().len());

            let changelogs = remote_library.changelogs();
            println!("Applying {} changelogs", changelogs.len());

            for changelog in changelogs {
                Self::apply_changelog(library, &changelog);
                // TODO this is redundant if the change has already been seen
                // and causes lots of slow.
                library.save(&changelog);
            }
        });

        // Upload a backup of the input library to storage.
        let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
        println!("Gathering local changes.");
        library.backup(temp_file.to_str().unwrap());
        let path = format!("{}/{}.db", self.path, library.id());
        let contents = std::fs::read(temp_file).unwrap();
        println!("Pushing local changes.");
        self.storage.put_object(&path, &contents);

        // Sync media files
        // TODO
        println!("Sync complete.");
    }

    fn apply_changelog(library: &Library, changelog: &ChangeLog) {
        let actor = changelog.actor.clone();
        let timestamp = changelog.timestamp.clone();
        let model = changelog.model.clone();
        let key = changelog.model_key.clone();
        let op = changelog.op.clone();
        // Ignore our own changes. Could be disabled to rebuild the db.
        if actor == library.id() {
            return
        }
        if model == "Track" {
            // TODO duplicated check of set in Track::apply_diff
            if op == "set" {
                let field = changelog.field.clone().unwrap();
                if let Some(newest_changelog) = library.find_newest_changelog_by_field(&model, &key, &field) {
                    if newest_changelog.timestamp >= timestamp {
                        return
                    }
                }
                let mut track = library.get(&key)
                    .or_else(|| Some(Track { key: Some(key), ..Default::default() })).unwrap();
                track.apply_diff(&[changelog.clone()]);
                library.save_unlogged(&track);
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

        let library1 = Library::open(":memory:");
        library1.save(&Track { 
            artist: Some("Grey Speaker".to_string()), 
            title: Some("One Thing".to_string()), 
            ..Default::default() 
        });
        sync.sync(&library1);

        let library2 = Library::open(":memory:");
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
        let sync = Sync::new(Box::new(storage), "TODO");

        let library = Library::open(":memory:");
        for i in 0..300 {
            library.save(&Track { 
                artist: Some(format!("Grey Speaker {}", i)), 
                title: Some(format!("One Thing {}", i)), 
                ..Default::default() 
            });
        }
        sync.sync(&library);

        let library2 = Library::open(":memory:");
        sync.sync(&library2);
    }
}


