pub mod ui;

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

// TODO desktop integration using souvlaki. currently broken on Windows.
// fn desktop_integration() {
//     #[cfg(not(target_os = "windows"))]
//     let hwnd = None;

//     #[cfg(target_os = "windows")]
//     let hwnd = {
//         use raw_window_handle::windows::WindowsHandle;

//         let handle: WindowsHandle = unimplemented!();
//         Some(handle.hwnd)
//     };

//     let config = PlatformConfig {
//         dbus_name: "dimple",
//         display_name: "Dimple",
//         hwnd,
//     };

//     let mut controls = MediaControls::new(config).unwrap();

//     // The closure must be Send and have a static lifetime.
//     controls
//         .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
//         .unwrap();

//     // Update the media metadata.
//     controls
//         .set_metadata(MediaMetadata {
//             title: Some("Time to get Dimply"),
//             artist: Some("Dimple"),
//             album: Some("Dimple Time"),
//             ..Default::default()
//         })
//         .unwrap();
// }

