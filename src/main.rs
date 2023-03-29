use std::sync::Arc;

use dimple::dimple::Dimple;
use eframe::egui::{self};
use rodio::{OutputStream, Sink};

// TODO BLOCKED make grid full width https://github.com/emilk/egui/discussions/1144#discussioncomment-2035457
// TODO make number of columns adapt to window width and tile size
// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs for smart playlist generation
// TODO how to load a custom font and use it globally https://github.com/catppuccin/egui/blob/main/examples/todo.rs#L77
// TODO info on how to do something on first frame: https://github.com/emilk/eframe_template/blob/master/src/app.rs#L24
// TODO escape should clear search
// TODO Continuous updates when downloading and loading libraries
// TODO search is sensitive to word order, i.e. infected shawarma vs. shawarma infected
// TODO parallelize the textures, although I think it might all happen on the
// first frame, in which case we could still do it somehow. Or just do whatever
// RetainedImage does.
// TODO Dashboard makes sense actually, and in fact, I can have as wild a tree
// as I want as long as:
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
        Box::new(|_cc| Box::new(Dimple::new(sink))),
    )
    .expect("eframe: pardon me, but no thank you");
}
