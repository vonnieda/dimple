use dimple::music_library::{local::LocalMusicLibrary, directory_library::DirectoryMusicLibrary, MusicLibrary};

fn main() {
    let source = DirectoryMusicLibrary::new("music");
    let dest = LocalMusicLibrary::new("data/library");

    let releases = source.releases();
    for release in releases {
        let title = release.title.clone();
        let artist = release.artist.as_ref().unwrap().clone();
        println!("Merging {} {}", artist, title);
        dest.merge_release(&release);
    }
}
