use std::sync::Arc;

use dimple_librarian::librarian::Librarian;

use directories::ProjectDirs;

fn main() {
    let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
    let dir = dirs.data_dir().to_str().unwrap();
    let librarian = Arc::new(Librarian::new(dir));
}
