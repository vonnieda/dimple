use std::sync::Arc;

use dimple_core::library::LibraryHandle;
use dimple_player::player::Player;
use dimple_ui_slint::{settings::Settings, librarian::Librarian, app_window_controller::AppWindowController};

fn main() -> Result<(), slint::PlatformError> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.parse_default_env();
    builder.init();
    log::debug!("Log initialized.");

    log::debug!("Loading settings.");
    let settings = Settings::default();

    log::debug!("Loading libraries.");
    let librarian: Arc<Librarian> = Arc::new(Librarian::from(settings.libraries));
    let library: LibraryHandle = librarian;

    log::debug!("Creating player.");
    let library_1 = library.clone();
    let player = Player::new(library_1);

    log::debug!("Initializing UI.");
    let ui = AppWindowController::new(library.clone(), player);

    log::info!("Running UI.");
    ui.run()
}

