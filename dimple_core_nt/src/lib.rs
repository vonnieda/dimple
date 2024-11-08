pub mod model;
pub mod library;
pub mod scanner;
pub mod play_queue;
pub mod player;
pub mod sync;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::{library::{self, Library}, player::Player, scanner::Scanner, sync::{memory_storage::MemoryStorage, s3_storage::S3Storage, Sync}};

// - [x] I can add MP3 and FLAC tracks by selecting a directory.
// 	- [x] Adding the same track twice should not duplicate it.
// 	- [x] Title, artist, album, length tags are stored as metadata for the file and shown in lists below.
// - [x] I can see a list of tracks in my library.
// 	- [x] Including title, artist, album, length.
// - [x] I can add tracks to the global play queue.
// - [x] I can list the global play queue.
// 	- [x] Including title, artist, album, length.
// - [x] I can start playing the play queue and it will play until completion.
// - [x] I can run the UI both on Mac and Windows.
// - [ ] I automatically sync library changes between my laptop and desktop using S3.
// 	- [ ] Files and metadata are uploaded to S3.
// 	- [ ] I can "Like" a track on my laptop and see the change reflected on Windows without manually refreshing.
// 		- [ ] The UI reacts to changes in the data store.
// 	- [ ] MP3 files added on laptop are also visible and playable on desktop, and vice-versa. Metadata should be synced immediately and files will be downloaded on demand.

    #[test]
    fn mvp() {
        // sync_path is the token to be shared between participants in the
        // sync. 
        let sync_path = Uuid::new_v4().to_string();
        let sync = Sync::new(Box::new(MemoryStorage::default()), &sync_path);

        {
            let library = Arc::new(Library::open(":memory:"));
            assert!(library.tracks().len() == 0);
            library.import(&Scanner::scan_directory("media_files"));
            assert!(library.tracks().len() > 0);
    
            let tracks = library.tracks();
            library.import(&Scanner::scan_directory("media_files"));
            assert!(tracks.len() == library.tracks().len());
    
            let player = Player::new(library.clone());
            assert!(player.play_queue().tracks.len() == 0);
    
            let tracks = library.tracks();
            let track = tracks.get(0).unwrap();
            player.play_queue_add(&track.key.clone().unwrap());
            assert!(player.play_queue().tracks.len() == 1);
            player.play_queue_add(&track.key.clone().unwrap());
            assert!(player.play_queue().tracks.len() == 2);

            sync.sync(&library);
        }

        {
            let library_2 = Arc::new(Library::open(":memory:"));
            assert!(library_2.tracks().len() == 0);
            sync.sync(&library_2);
            assert!(library_2.tracks().len() > 0);
        }
    }
}
