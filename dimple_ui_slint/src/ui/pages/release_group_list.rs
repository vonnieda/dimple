use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Entity;
use dimple_core::model::ReleaseGroup;
use slint::Model;
use slint::ModelRc;

pub fn release_group_list(app: &App) {
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {
        let mut release_groups: Vec<ReleaseGroup> = librarian
            .list(&ReleaseGroup::default().into(), &None)
            .unwrap()
            .map(Into::into)
            .collect();
        release_groups.sort_by_key(|a| a.title.clone().unwrap_or_default().to_lowercase());

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = release_groups.iter().cloned().enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_release_group_list().cards.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_release_group_list().cards.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_release_group_list(adapter);
            ui.set_page(Page::ReleaseGroupList);
        }).unwrap();
    });
}
