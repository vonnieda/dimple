use dimple_core::{model::DimpleReleaseGroup, library::{Library, LibraryEntity}};
use dimple_deezer_library::DeezerLibrary;
use dimple_librarian::librarian::Librarian;
use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_wikidata_library::WikidataLibrary;

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();
    builder.filter(Some("symphonia_core::probe"), log::LevelFilter::Off);
    builder.init();

    let libr = Librarian::new("sources_tmp_data");
    libr.add_library(Box::<MusicBrainzLibrary>::default());
    libr.add_library(Box::<DeezerLibrary>::default());
    libr.add_library(Box::<WikidataLibrary>::default());
    let rg = DimpleReleaseGroup::get("68ee0e21-a7c2-467d-8808-fd38fec2ffb8", &libr).unwrap();
    // let sources = libr.sources(&LibraryEntity::ReleaseGroup(rg));
    // for source in sources {
    //     dbg!(source);
    // }
}
