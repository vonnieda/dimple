pub mod model;
pub mod library;
pub mod scanner;
pub mod play_queue;
pub mod player;

#[cfg(test)]
mod tests {
    use crate::{library::Library, scanner::Scanner};

    #[test]
    fn mvp() {
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

    #[test]
    fn bug_uuid_changes_on_reimport() {
        todo!()
    }
}
