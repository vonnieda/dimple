use crate::ui::app_window_controller::App;
use crate::ui::Page;

pub fn skeleton_list_init(app: &App) {
}

pub fn skeleton_list(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.set_page(Page::ReleaseList);
        })
        .unwrap();
    });
}
