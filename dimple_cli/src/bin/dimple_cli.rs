use std::time::Duration;

use dimple_core::{collection::Collection, model::{Artist, Model, Recording}};
use dimple_coverartarchive_library::CoverArtArchiveLibrary;
use dimple_deezer_library::DeezerLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_file_library::dimple_file_library::FileLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_librarian::librarian::Librarian;
use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_theaudiodb_library::TheAudioDbLibrary;
use dimple_wikidata_library::WikidataLibrary;
use directories::ProjectDirs;

fn main() -> anyhow::Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();
    builder.filter(Some("symphonia_core"), log::LevelFilter::Off);
    builder.filter(Some("symphonia_metadata"), log::LevelFilter::Off);
    builder.filter(Some("symphonia_bundle_mp3"), log::LevelFilter::Off);
    builder.filter(Some("symphonia_format_isomp4"), log::LevelFilter::Off);
    builder.init();


    let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
    let dir = dirs.data_dir().to_str().unwrap();
    let librarian = Librarian::new(dir);

    let paths = vec![
        "/Users/jason/Music/My Music".to_string(),
    ];
    // librarian.add_library(Box::new(FileLibrary::new(&paths)));
    librarian.add_library(Box::<MusicBrainzLibrary>::default());
    // librarian.add_library(Box::<TheAudioDbLibrary>::default());
    // librarian.add_library(Box::<FanartTvLibrary>::default());
    // librarian.add_library(Box::<DeezerLibrary>::default());
    // librarian.add_library(Box::<WikidataLibrary>::default());
    // librarian.add_library(Box::<LastFmLibrary>::default());
    // librarian.add_library(Box::<CoverArtArchiveLibrary>::default());

    // std::thread::sleep(Duration::from_secs(5));

    // for artist in Artist::list(&librarian) {
    //     dbg!(artist.name);
    // }
    // for track in Recording::list(&librarian) {
    //     dbg!(track.title);
    // }

    let artists: Vec<Artist> = Artist::search("we were heading north", &librarian).collect();
    dbg!(&artists);

    let artist = artists.first().unwrap();
    dbg!(&artist);

    let artist = artist.fetch(&librarian).unwrap();
    dbg!(&artist);

    // let release_groups: Vec<_> = artist.release_groups(&librarian).collect();
    // dbg!(&release_groups);

    // for r in release_groups {
    //     let rg = r.fetch(&librarian).unwrap();
    //     dbg!(&rg);
    // }

    Ok(())
}
