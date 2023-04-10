use std::sync::Arc;

use dimple::dimple::Dimple;
use eframe::egui::{self};
use rodio::{OutputStream, Sink};

// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs for smart playlist generation
// TODO how to load a custom font and use it globally https://github.com/catppuccin/egui/blob/main/examples/todo.rs#L77
// TODO escape should clear search
// TODO search is sensitive to word order, i.e. infected shawarma vs. shawarma 
//      infected
// TODO Dashboard makes sense actually, and in fact, I can have as wild a tree
//      as I want as long as:
// 1. Clicking anything updates the search bar with the terms needed to get to
//    where we are.
// 2. Hitting escape clears search.
// Dashboard can contain "Artists", "Albums", "For You", "Today In", "Genres",
// etc. A bunch of derived stuff. And then scrolling down can include favorites
// and recents and such.
// TODO Clicking on something should NEVER suddenly play that thing and clear
// the queue. The queue is precious.
// TODO build app for Mac: 
//   https://www.bugsnag.com/blog/building-macos-apps-with-rust
//   https://agmprojects.com/blog/packaging-a-game-for-windows-mac-and-linux-with-rust.html
// TODO all these little caches I keep writing should probably just be converted to
// a single "thing" like redis

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1440.0, 1024.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Dimple",
        native_options,
        Box::new(|cc| Box::new(Dimple::new(cc, sink))),
    )
    .expect("eframe: pardon me, but no thank you");
}
