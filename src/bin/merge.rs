use dimple::music_library::{local::LocalMusicLibrary, Library};

fn main() {
    let source = LocalMusicLibrary::new("data/library");
    let dest = LocalMusicLibrary::new("data/library2");

    let releases = source.releases();
    for release in releases {
        let title = release.title.clone();
        let artist = release.artist.as_ref().unwrap().clone();
        println!("Merging {} {}", artist, title);
        dest.merge_release(&release);
    }
}
