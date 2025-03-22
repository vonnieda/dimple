use std::time::{Duration, Instant};

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

	let song_filename = std::env::args()
		.nth(1)
		.expect("Expected one filename as the first argument.");
	let player = Player::new(None)?;
	info!("Loading song '{}'...", song_filename);
	let song = Song::from_file(&song_filename, None)?;
	player.play_song_next(&song, None)?;
	let start = Instant::now();
	info!("Playing.");
	while player.has_current_song() {
		let playback_speed = (start.elapsed().as_secs_f64()).sin() / 5.0 + 1.0;
		player.set_playback_speed(playback_speed);
		info!("Playback speed now: {playback_speed}");
		std::thread::sleep(Duration::from_millis(10));
	}
	info!("Exiting.");

	Ok(())
}
