use dimple_core::library::Library;
use dimple_folder_library::folder_library::FolderLibrary;

fn main() {
    let library = FolderLibrary::new("/Users/jason/Music/My Music");
    library.releases();
}
