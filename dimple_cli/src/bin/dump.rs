use dimple_musicbrainz_library::musicbrainz_library::MusicBrainzLibrary;
use dimple_core::library::Library;

fn main() {
    let library = MusicBrainzLibrary::new();

    for result in library.search("Metallica") {
        dbg!(result);
    }
}
