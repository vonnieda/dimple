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
/// library_uuid.db
/// library_uuid.db/remote_uuid.db
/// library_uuid.db/remote_2_uuid.db
/// 
/// Okay, so config options:
/// 
/// Laptop: 1
/// Desktop: 2
/// Mobile: 3
/// 
/// 1 -> 1.db/1.db
/// 2 -> 1.db/2.db
/// 3 -> 1.db/3.db

pub mod storage;
pub mod s3_storage;
pub mod memory_storage;

use storage::Storage;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{library::{Library, LibraryModel}, model::{ChangeLog, Track}};

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
        println!("Synchronizing {}", library.uuid());
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
                changelog.save(library, true);
            }
        });

        // Upload a backup of the input library to storage.
        let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
        library.backup(temp_file.to_str().unwrap());
        let path = format!("{}/{}.db", self.path, library.uuid());
        let contents = std::fs::read(temp_file).unwrap();
        self.storage.put_object(&path, &contents);

        // Sync media files
        // TODO
    }

    fn apply_changelog(library: &Library, changelog: &ChangeLog) {
        let actor = changelog.actor.clone();
        let timestamp = changelog.timestamp.clone();
        let model = changelog.model.clone();
        let key = changelog.key.clone();
        let op = changelog.op.clone();
        // Ignore our own changes. Could be disabled to rebuild the db.
        if actor == library.uuid() {
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
                let mut track = Track::get(library, &key)
                    .or_else(|| Some(Track { key: Some(key), ..Default::default() })).unwrap();
                track.apply_diff(&[changelog.clone()]);
                // TODO this should not create a changelog, but it does. need a flag
                // on save, I guess.
                track.save(library, false);
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
        let sync = Sync::new(Box::new(storage), "TODO");

        let library1 = Library::open(":memory:");
        Track { 
            artist: Some("Grey Speaker".to_string()), 
            title: Some("One Thing".to_string()), 
            path: Library::uuid_v4().to_string(),
            ..Default::default() 
        }.save(&library1, true);
        sync.sync(&library1);

        let library2 = Library::open(":memory:");
        Track { 
            title: Some("Tall Glass".to_string()), 
            path: Library::uuid_v4().to_string(),
            ..Default::default() 
        }.save(&library2, true);
        sync.sync(&library2);

        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);
        sync.sync(&library2);
        sync.sync(&library1);

        assert!(library1.tracks().len() == 2);
        assert!(library2.tracks().len() == 2);
        assert!(library1.changelogs().len() == 10);
        assert!(library2.changelogs().len() == 10);
        assert!(library1.changelogs() == library2.changelogs());
    }

    #[test]
    fn big_library() {
        let storage = MemoryStorage::default();
        let sync = Sync::new(Box::new(storage), "TODO");

        let library = Library::open(":memory:");
        for i in 0..300 {
            Track { 
                artist: Some(format!("Grey Speaker {}", i)), 
                title: Some(format!("One Thing {}", i)), 
                ..Default::default() }.save(&library, true);
        }
        sync.sync(&library);

        let library2 = Library::open(":memory:");
        sync.sync(&library2);

        dbg!(library.changelogs().len());
        dbg!(library2.changelogs().len());
        dbg!(library.tracks().len());
        dbg!(library2.tracks().len());
    }
}


