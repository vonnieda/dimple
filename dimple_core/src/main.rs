use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};
use dimple_core::{
    import::spotify,
    library::Library,
    model::{Artist, ModelBasics as _, Release, Track},
    player::Player,
};
use directories::ProjectDirs;

#[derive(Parser)]
#[command(name = "dimple")]
#[command(about = "A music library management and playback tool", long_about = None)]
#[command(version)]
struct Cli {
    /// Override the default data directory (env: DIMPLE_ROOT)
    #[arg(long, global = true, env = "DIMPLE_ROOT")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import tracks from a file or directory
    Import {
        /// Path to the file or directory to import
        path: PathBuf,
    },
    /// Import Spotify data
    ImportSpotify {
        /// Path to the Spotify data file
        path: PathBuf,
    },
    /// List all artists in the library
    Artists,
    /// List all releases in the library
    Releases,
    /// List all tracks in the library
    Tracks,
    /// Search for artists, releases, or tracks
    Search {
        /// Search query
        query: String,
    },
    /// Toggle 'liked' status for a track
    Like {
        /// Track ID (UUID)
        track_id: String,
    },
    /// List tracks in the play queue
    Queue,
    /// Add a track to the play queue
    Add {
        /// Track ID (UUID)
        track_id: String,
    },
    /// Remove a track from the queue by position
    Remove {
        /// Position in the queue (1-based index)
        position: usize,
    },
    /// Clear the play queue
    Clear,
    /// Play the queue from start to finish
    Play,
    /// Sync the library with an S3 target
    Sync,
}

fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();
    let library = initialize_library(cli.root.as_deref())?;
    let player = Player::new(library.clone());

    match cli.command {
        Commands::Import { path } => {
            if !path.exists() {
                anyhow::bail!("Path does not exist: {}", path.display());
            }

            println!(
                "Library currently contains {} tracks.",
                Track::list(&library).len()
            );
            println!("Importing {}...", path.display());

            library.import(
                path.to_str()
                    .with_context(|| format!("Invalid path encoding: {}", path.display()))?,
            );

            println!(
                "Import complete. Library now contains {} tracks, {} releases, {} artists.",
                Track::list(&library).len(),
                Release::list(&library).len(),
                Artist::list(&library).len()
            );
        }
        Commands::ImportSpotify { path } => {
            if !path.exists() {
                anyhow::bail!("Path does not exist: {}", path.display());
            }

            println!("Importing Spotify data from {}...", path.display());
            spotify::import(
                &library,
                path.to_str()
                    .with_context(|| format!("Invalid path encoding: {}", path.display()))?,
            );
            println!("Spotify import complete.");
        }
        Commands::Artists => {
            let artists = Artist::list(&library);
            print_artists(&artists);
        }
        Commands::Releases => {
            let releases = Release::list(&library);
            print_releases(&library, &releases);
        }
        Commands::Tracks => {
            let tracks = Track::list(&library);
            print_tracks(&library, &tracks);
        }
        Commands::Search { query } => {
            search_library(&library, &query);
        }
        Commands::Like { track_id } => {
            let _track = Track::get(&library, &track_id).with_context(|| {
                format!("Track not found with ID: {}", track_id)
            })?;
            // TODO: Implement like toggle functionality
            println!("Like toggle not yet implemented for track: {}", track_id);
        }
        Commands::Queue => {
            let tracks = player.queue().tracks(&library);
            print_queue(&library, &tracks);
        }
        Commands::Add { track_id } => {
            let track = Track::get(&library, &track_id).with_context(|| {
                format!("Track not found with ID: {}", track_id)
            })?;

            let track_title = track.title.clone().unwrap_or_else(|| "Unknown".to_string());
            let artist = track.artist_name(&library).unwrap_or_else(|| "Unknown".to_string());

            player.queue().append(&library, &track.into());
            println!("Added to queue: {} - {}", artist, track_title);

            let tracks = player.queue().tracks(&library);
            print_queue(&library, &tracks);
        }
        Commands::Remove { position } => {
            let tracks = player.queue().tracks(&library);
            if position == 0 || position > tracks.len() {
                anyhow::bail!(
                    "Invalid position {}. Queue has {} tracks (use 1-based index).",
                    position,
                    tracks.len()
                );
            }
            // TODO: Implement actual queue removal - needs a method on queue
            println!("Remove not yet implemented. Position: {}", position);
        }
        Commands::Clear => {
            player.queue().clear(&library);
            println!("Queue cleared.");
        }
        Commands::Play => {
            player.play();
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Commands::Sync => {
            library.sync();
        }
    }

    Ok(())
}

fn init_logging() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();

    // Filter out noisy Symphonia logs
    builder.filter(Some("symphonia_core"), log::LevelFilter::Off);
    builder.filter(Some("symphonia_bundle_mp3"), log::LevelFilter::Off);
    builder.filter(Some("symphonia_metadata"), log::LevelFilter::Off);
    builder.filter(Some("symphonia_format_isomp4"), log::LevelFilter::Off);
    builder.filter(Some("tiny_skia::painter"), log::LevelFilter::Off);

    builder.init();
}

fn initialize_library(root: Option<&std::path::Path>) -> Result<Arc<Library>> {
    let dirs = ProjectDirs::from("lol", "Dimple", "dimple_ui_slint")
        .context("Failed to determine project directories")?;

    let (data_dir, config_dir, cache_dir) = if let Some(root) = root {
        (
            root.join("data"),
            root.join("config"),
            root.join("cache"),
        )
    } else {
        (
            dirs.data_dir().to_path_buf(),
            dirs.config_dir().to_path_buf(),
            dirs.cache_dir().to_path_buf(),
        )
    };

    let library_path = data_dir.join("library.db");
    let _config_path = config_dir.join("config.db");
    let image_cache_dir = cache_dir.join("image_cache");

    std::fs::create_dir_all(&data_dir)
        .context("Failed to create data directory")?;
    std::fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    std::fs::create_dir_all(&cache_dir)
        .context("Failed to create cache directory")?;
    std::fs::create_dir_all(&image_cache_dir)
        .context("Failed to create image cache directory")?;

    let library = Arc::new(Library::open(
        library_path
            .to_str()
            .context("Invalid library path encoding")?,
    ));

    Ok(library)
}

fn print_artists(artists: &[Artist]) {
    if artists.is_empty() {
        println!("No artists found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Name"]);

    for artist in artists {
        table.add_row(vec![
            artist.id.clone().unwrap_or_default(),
            artist.name.clone().unwrap_or_default(),
        ]);
    }

    println!("{table}");
}

fn print_releases(library: &Library, releases: &[Release]) {
    if releases.is_empty() {
        println!("No releases found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Title", "Artist"]);

    for release in releases {
        table.add_row(vec![
            release.id.clone().unwrap_or_default(),
            release.title.clone().unwrap_or_default(),
            release.artist_name(library).unwrap_or_default(),
        ]);
    }

    println!("{table}");
}

fn print_tracks(library: &Library, tracks: &[Track]) {
    if tracks.is_empty() {
        println!("No tracks found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Artist", "Album", "Title"]);

    for track in tracks {
        table.add_row(vec![
            track.id.clone().unwrap_or_default(),
            track.artist_name(library).unwrap_or_default(),
            track.album_name(library).unwrap_or_default(),
            track.title.clone().unwrap_or_default(),
        ]);
    }

    println!("{table}");
}

fn print_queue(library: &Library, tracks: &[Track]) {
    if tracks.is_empty() {
        println!("Queue is empty.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["#", "ID", "Artist", "Album", "Title"]);

    for (i, track) in tracks.iter().enumerate() {
        table.add_row(vec![
            (i + 1).to_string(),
            track.id.clone().unwrap_or_default(),
            track.artist_name(library).unwrap_or_default(),
            track.album_name(library).unwrap_or_default(),
            track.title.clone().unwrap_or_default(),
        ]);
    }

    println!("{table}");
    println!("Total: {} track(s)", tracks.len());
}

fn search_library(library: &Library, query: &str) {
    let query_lower = query.to_lowercase();

    // Search artists
    let artists: Vec<_> = Artist::list(library)
        .into_iter()
        .filter(|a| {
            a.name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
        })
        .collect();

    if !artists.is_empty() {
        println!("\n=== Artists ({}) ===", artists.len());
        print_artists(&artists);
    }

    // Search releases
    let releases: Vec<_> = Release::list(library)
        .into_iter()
        .filter(|r| {
            r.title
                .as_ref()
                .map(|t| t.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
                || r.artist_name(library)
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .collect();

    if !releases.is_empty() {
        println!("\n=== Releases ({}) ===", releases.len());
        print_releases(library, &releases);
    }

    // Search tracks
    let tracks: Vec<_> = Track::list(library)
        .into_iter()
        .filter(|t| {
            t.title
                .as_ref()
                .map(|title| title.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
                || t.artist_name(library)
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || t.album_name(library)
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .collect();

    if !tracks.is_empty() {
        println!("\n=== Tracks ({}) ===", tracks.len());
        print_tracks(library, &tracks);
    }

    if artists.is_empty() && releases.is_empty() && tracks.is_empty() {
        println!("No results found for '{}'", query);
    }
}
