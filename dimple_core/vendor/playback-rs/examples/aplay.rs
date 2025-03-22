use color_eyre::eyre::Result;
use log::info;

use playback_rs::{Player, Song};

fn main() -> Result<()> {
	color_eyre::install()?;
	simplelog::TermLogger::init(
		simplelog::LevelFilter::Trace,
		simplelog::Config::default(),
		simplelog::TerminalMode::Mixed,
		simplelog::ColorChoice::Auto,
	)
	.unwrap();

	let filenames = std::env::args().skip(1);
	let player = Player::new(None)?;
	for next_song in filenames {
		info!("Loading song '{}'...", next_song);
		let song = Song::from_file(&next_song, None)?;
		info!("Waiting for queue space to become available...");
		while player.has_next_song() {
			std::thread::sleep(std::time::Duration::from_millis(100));
		}
		info!(
			"Queueing next song '{}' with {:?} left in current song...",
			next_song,
			player.get_playback_position()
		);
		player.play_song_next(&song, None)?;
		info!("Queued.");
	}
	info!("Waiting for songs to finish.");
	while player.has_current_song() {
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	info!("Exiting.");

	Ok(())
}
