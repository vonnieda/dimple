pub mod ui;
pub mod config;

use ui::app_window_controller::AppWindowController;

fn main() -> Result<(), slint::PlatformError> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();

    // [2024-01-24T21:03:58.412Z INFO  symphonia_core::probe] found the format marker [49, 44, 33] @ 0+2 bytes.
    builder.filter(Some("symphonia_core"), log::LevelFilter::Off);

    // [2024-01-24T21:02:27.904Z INFO  symphonia_bundle_mp3::demuxer] estimating duration from bitrate, may be inaccurate for vbr files
    builder.filter(Some("symphonia_bundle_mp3"), log::LevelFilter::Off);

    // [2024-01-24T21:02:27.905Z INFO  symphonia_metadata::id3v2] unsupported frame UFID
    builder.filter(Some("symphonia_metadata"), log::LevelFilter::Off);

    // [2024-01-24T21:06:24.917Z INFO  symphonia_format_isomp4::demuxer] stream is seekable with len=3037538 bytes.
    builder.filter(Some("symphonia_format_isomp4"), log::LevelFilter::Off);

    builder.init();

    let ui = AppWindowController::new();
    ui.run()
}

