use dimple_ui_slint::app_window_controller::AppWindowController;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};

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


    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        use raw_window_handle::windows::WindowsHandle;

        let handle: WindowsHandle = unimplemented!();
        Some(handle.hwnd)
    };

    let config = PlatformConfig {
        dbus_name: "my_player",
        display_name: "My Player",
        hwnd,
    };

    let mut controls = MediaControls::new(config).unwrap();

    // The closure must be Send and have a static lifetime.
    controls
        .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
        .unwrap();

    // Update the media metadata.
    controls
        .set_metadata(MediaMetadata {
            title: Some("Time to get Dimply"),
            artist: Some("Dimple"),
            album: Some("Dimple Time"),
            ..Default::default()
        })
        .unwrap();

    // // Your actual logic goes here.
    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    // }

    let ui = AppWindowController::default();
    ui.run()
}
