

use std::time::Duration;

use dimple_core::model::Artist;
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

    let librarian = default_librarian();
    let paths = vec![
        "/Users/jason/Music/My Music/We Were Heading North".to_string(),
    ];
    librarian.add_library(Box::new(FileLibrary::new(&paths)));
    std::thread::sleep(Duration::from_secs(2));

    let artist = Artist::search("we were heading north", &librarian).next().expect("no search results");
    // let artist = Artist::list(&librarian).next().expect("no artists");
    log::info!("{:?}", &artist);
    let release = artist.releases(&librarian).next().expect("no releases");
    log::info!("{:?}", &release);
    let recording = release.recordings(&librarian).next().expect("no recordings");
    log::info!("{:?}", &recording);
    let source = recording.sources(&librarian).next();
    log::info!("{:?}", source);

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
