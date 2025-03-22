use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use log::{debug, error};

fn main() -> Result<()> {
	color_eyre::install()?;
	simplelog::TermLogger::init(
		simplelog::LevelFilter::Trace,
		simplelog::Config::default(),
		simplelog::TerminalMode::Mixed,
		simplelog::ColorChoice::Auto,
	)
	.unwrap();
	let listener = TcpListener::bind("0.0.0.0:3305")?;

	debug!("Listening for clients...");

	let start_time = Arc::new(Instant::now());
	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				debug!("Got connection!");
				let start_time = start_time.clone();
				thread::spawn(move || loop {
					debug!("Sending ping...");
					let mut buf = [0; 16];
					let before = start_time.elapsed().as_micros() as i128;
					stream
						.write_all(&[&[0][..], &before.to_be_bytes()[..]].concat())
						.unwrap();
					stream.read_exact(&mut buf).unwrap();
					let after = start_time.elapsed().as_micros() as i128;
					let client_time = i128::from_be_bytes(buf);
					let adjustment = (before + after) / 2 - client_time;
					debug!(
						"Client time: {}/{}, Old server time: {}, New server time: {}/{}, Calculated adjustment: {}",
						client_time,
						client_time-before,
						before,
						after,
						after-before,
						adjustment,
					);
					stream
						.write_all(&[&[1][..], &adjustment.to_be_bytes()[..]].concat())
						.unwrap();
					thread::sleep(Duration::from_secs(1));
				});
			}
			Err(e) => {
				error!("Receiving connection failed: {e}");
			}
		}
	}
	Ok(())
}
