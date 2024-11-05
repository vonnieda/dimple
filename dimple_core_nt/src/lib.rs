pub mod model;
pub mod library;
pub mod scanner;
pub mod play_queue;
pub mod player;
pub mod sync;

#[cfg(test)]
mod tests {
    use crate::{library::{self, Library}, scanner::Scanner, sync::{s3_storage::S3Storage, Sync}};

    #[test]
    fn mvp() {
        let library_1 = Library::open(":memory:");
        let media_files_1 = Scanner::scan_directory("media_files");
        library_1.import(&media_files_1);
        let s3_storage_1 = S3Storage::default();
        let sync_1 = Sync::new(Box::new(s3_storage_1), &library_1.uuid());
        sync_1.sync(&library_1);

        let library_2 = Library::open(":memory:");
        let s3_storage_2 = S3Storage::default();
        let sync_2 = Sync::new(Box::new(s3_storage_2), &library_1.uuid());
        sync_2.sync(&library_2);

        assert!(library_1.tracks().len() == library_2.tracks().len());

        sync_1.sync(&library_1);
        sync_2.sync(&library_2);
        sync_1.sync(&library_1);
        sync_2.sync(&library_2);

        assert!(library_1.tracks().len() == library_2.tracks().len());
        assert!(library_1.changelogs().len() == library_2.changelogs().len());
    }

    #[test]
    fn does_not_duplicate_on_import() {
        let library = Library::open(":memory:");

        let media_files = Scanner::scan_directory("media_files");
        assert!(media_files.len() > 0);

        library.import(&media_files);
        let tracks1 = library.tracks();
        assert!(tracks1.len() > 0);
        dbg!(tracks1.len());

        library.import(&media_files);
        let tracks2 = library.tracks();
        assert!(tracks2.len() == tracks1.len());
        dbg!(tracks2.len());
    }
}
