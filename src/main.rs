// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs for smart playlist generation
// TODO escape should clear search
// TODO search is sensitive to word order, i.e. infected shawarma vs. shawarma 
//      infected
// TODO dashboard - card grid with auto generated moods, playlists, etc.
// TODO never clear the queue in a single action, such as clicking a track.
// TODO build app for Mac: 
//   https://www.bugsnag.com/blog/building-macos-apps-with-rust
//   https://agmprojects.com/blog/packaging-a-game-for-windows-mac-and-linux-with-rust.html
// TODO all these little caches I keep writing should probably just be converted to
// a single "thing" like redis
// TODO figure out a better way to render the svgs - they look like trash
// TODO app icon https://github.com/emilk/egui/discussions/1574
//      https://github.com/KunalBagaria/redock
// TODO test gapless playback: Us and Them -> And Colour You Like
// TODO I think I can drop the state from almost all of the components now
// that Theme is in the ui. Try it - would simplify a ton.

use std::sync::Arc;

use dimple::dimple::Dimple;
use eframe::egui::{self};
use rodio::{OutputStream, Sink};

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());

    let native_options = eframe::NativeOptions {
        resizable: true,
        initial_window_size: Some(egui::vec2(1440.0, 1024.0)),
        // initial_window_size: Some(egui::vec2(1024.0, 720.0)),
        min_window_size: Some(egui::Vec2 { x: 525.0, y: 575.0 }),
        ..Default::default()
    };

    // native_options.set_window_icon_from("./test.png");

    eframe::run_native(
        "Dimple",
        native_options,
        Box::new(|cc| Box::new(Dimple::new(cc, sink))),
    )
    .expect("eframe: pardon me, but no thank you");
}
