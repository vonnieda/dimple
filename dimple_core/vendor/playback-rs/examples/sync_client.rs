use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use log::{debug, info, warn};

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
	debug!("Connecting...");
	let mut stream = TcpStream::connect("192.168.2.136:3305")?;
	debug!("Connected!");

	let start_time = Arc::new(Instant::now());
	let adjustment: Arc<RwLock<i128>> = Arc::new(RwLock::new(0));

	{
		let start_time = start_time.clone();
		let adjustment = adjustment.clone();
		thread::spawn(move || {
			let mut last_adjustment_micros: i128 = 0;
			loop {
				let mut buf = [0; 17];
				debug!("Waiting for message...");
				stream.read_exact(&mut buf).unwrap();
				match buf[0] {
					0 => {
						// Ping value
						let cur_time = start_time.elapsed().as_micros();
						stream.write_all(&cur_time.to_be_bytes()).unwrap();
						let server_time = u128::from_be_bytes(buf[1..].try_into().unwrap());
						debug!("Sent time: {}, Server time: {}", cur_time, server_time,);
					}
					1 => {
						// Adjustment value
						let new_adjustment_micros =
							i128::from_be_bytes(buf[1..].try_into().unwrap());
						debug!("Got new time adjustment value: {new_adjustment_micros}");
						let mut adjustment_micros = adjustment.write().unwrap();
						let diff = (*adjustment_micros - new_adjustment_micros).abs();
						let last_diff = (last_adjustment_micros - new_adjustment_micros).abs();
						if diff > 10000 && last_diff > 100000 {
							warn!("Adjustment changed very quickly (+/-{diff}, compared to last: +/-{last_diff}) and is likely eronius, ignoring for now...");
						} else {
							*adjustment_micros = new_adjustment_micros;
						}
						last_adjustment_micros = new_adjustment_micros;
					}
					n => {
						warn!("Unknown server message: {n}");
					}
				}
			}
		});
	}
	let player = Player::new(None)?;
	let mut filenames = std::env::args().skip(1).peekable();
	let mut seek_next = true;
	while player.has_current_song() || filenames.peek().is_some() {
		if !player.has_next_song() {
			if let Some(filename) = filenames.next() {
				info!("Loading next song...");
				let song = Song::from_file(&filename, None)?;
				player.play_song_next(&song, None)?;
				info!("Song queued.");
			}
		}
		let adjustment = *adjustment.read().unwrap();
		let current_time = start_time.elapsed().as_micros() as i128 + adjustment;
		let playback_micros = player
			.get_playback_position()
			.map(|(pp, _)| pp.as_micros() as i128)
			.unwrap_or(0);
		let time_diff = current_time - playback_micros;
		let speedup = (1.0 + time_diff as f64 / 10000000.0).clamp(0.99, 1.01);
		info!("Current song time: {}µs, current (sync) time: {}µs, time diff: {:.2}s, speedup: {:.2}x, current adjustment: {}µs", playback_micros, current_time, time_diff as f64 / 1000000.0, speedup, adjustment);

		if time_diff > 1000000 || seek_next {
			if !seek_next {
				warn!("Time drift is larger than 1 second! Seeking...");
			}
			player.seek(Duration::from_micros(current_time.try_into()?));
			seek_next = !seek_next;
		} else {
			player.set_playback_speed(speedup);
		}

		thread::sleep(Duration::from_millis(100));
	}
	Ok(())
}
