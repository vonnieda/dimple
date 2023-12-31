enum LibraryItem {
    Artist(String),
    Release(String),
    Track(String),
}

trait Library {
    fn name(&self) -> String;
    fn search(&self) -> Box<dyn Iterator<Item = LibraryItem>>;
}

trait LibrarySearch {
}

#[derive(Default)]
struct FileLibrary {

}

impl Library for FileLibrary {
    fn name(&self) -> String {
        "hi".to_string()
    }

    fn search(&self) -> Box<dyn Iterator<Item = LibraryItem>> {
        let r: Vec<LibraryItem> = vec![];
        Box::new(r.into_iter())
    }
}


fn main() {
    let lib: Box<dyn Library> = Box::new(FileLibrary::default());
    lib.name();
    lib.search();
}
