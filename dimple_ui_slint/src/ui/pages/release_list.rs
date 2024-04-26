use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Entity;
use dimple_core::model::Release;
use slint::Model;
use slint::ModelRc;

pub fn release_list(app: &App) {
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {
        let mut releases: Vec<Release> = librarian
            .list(&Release::default().into(), None)
            .unwrap()
            .map(Into::into)
            .collect();
        releases.sort_by_key(|a| a.date.clone().unwrap_or_default().to_lowercase());

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = releases.iter().cloned().enumerate()
                .map(|(index, release)| {
                    let mut card: CardAdapter = release.clone().into();
                    card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_release_list().cards.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_release_list().cards.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_release_list(adapter);
            ui.set_page(Page::ReleaseList);
        }).unwrap();
    });
}
