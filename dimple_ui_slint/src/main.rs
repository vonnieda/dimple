use dimple_ui_slint::app_window_controller::AppWindowController;

fn main() -> Result<(), slint::PlatformError> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();
    builder.filter(Some("symphonia_core::probe"), log::LevelFilter::Off);
    builder.init();

    let ui = AppWindowController::default();
    ui.run()
}
