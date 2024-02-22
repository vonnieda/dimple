

use std::{sync::Arc, thread, time::Duration};

use dimple_core::{collection::Collection, model::{Artist, Entity, Recording}};
use dimple_coverartarchive_library::CoverArtArchiveLibrary;
use dimple_deezer_library::DeezerLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_file_library::dimple_file_library::FileLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_librarian::librarian::Librarian;
use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_player::player::Player;
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

    let librarian = Arc::new(default_librarian());
    let paths = vec![
        "/Users/jason/Music/Dimple Test Tracks".to_string(),
        // "/Users/jason/Music/My Music/We Were Heading North".to_string(),
        // "/Users/jason/Music/My Music/Metallica".to_string(),
        // "/Users/jason/Music/My Music/Megadeth".to_string(),
        // "/Users/jason/Music/My Music/Opeth".to_string(),
        // "/Users/jason/Music/My Music/Fen".to_string(),
    ];
    librarian.add_library(Box::new(FileLibrary::new(&paths)));
    std::thread::sleep(Duration::from_secs(1));


    // let artist_count = Artist::list(&librarian).count();
    // for (i, artist) in Artist::list(&librarian).enumerate() {
    //     log::info!("Artist {}/{}: {} (mbid:{:?})", 
    //         i + 1, artist_count,
    //         artist.name.clone().unwrap_or_default(),
    //         artist.mbid().unwrap_or_default());
    //     for release in artist.releases(&librarian) {
    //         log::info!("    Release: {}", release.title.clone().unwrap_or_default());
    //         for recording in release.recordings(&librarian) {
    //             log::info!("        Recording: {}", recording.title.clone().unwrap_or_default());
    //             for source in recording.sources(&librarian) {
    //                 log::info!("            Source: {}", source.key.unwrap_or_default());
    //             }
    //         }
    //     }
    // }

    let player = Player::new(librarian.clone());
    let recording = librarian.list(&Recording::default().entity(), None).next().unwrap();
    dbg!(&recording);
    player.enqueue(&recording);
    player.play();
    thread::sleep(Duration::from_secs(5));
    player.pause();
    thread::sleep(Duration::from_secs(5));
    player.play();
    thread::sleep(Duration::from_secs(5));
    player.stop();

    loop {
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn default_librarian() -> Librarian {
    let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
    let dir = dirs.data_dir().to_str().unwrap();
    let librarian = Librarian::new(dir);
    librarian.add_library(Box::<MusicBrainzLibrary>::default());
    librarian.add_library(Box::<TheAudioDbLibrary>::default());
    librarian.add_library(Box::<FanartTvLibrary>::default());
    librarian.add_library(Box::<DeezerLibrary>::default());
    librarian.add_library(Box::<WikidataLibrary>::default());
    librarian.add_library(Box::<LastFmLibrary>::default());
    librarian.add_library(Box::<CoverArtArchiveLibrary>::default());
    librarian
}
