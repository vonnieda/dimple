use crate::ui::images::lazy_load_images;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_librarian::librarian::Librarian;
use slint::ComponentHandle;
use slint::ModelRc;

pub fn artist_list(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let librarian = librarian.clone();
    let ui = ui.clone();
    std::thread::spawn(move || {
        let mut artists: Vec<Artist> = librarian
            .list(&Artist::default().into(), None)
            .unwrap()
            .map(Into::into)
            .collect();
        artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = artists.iter().cloned().map(Into::into).collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_artist_list(adapter);
            ui.set_page(Page::ArtistList);
            // It again seems like the place to do this is in the mapping from artist
            // to card.
            let models: Vec<dimple_core::model::Model> = artists.iter().cloned().map(Into::into).collect();
            lazy_load_images(&librarian, models.as_slice(), ui.as_weak(), |ui| ui.get_artist_list().cards);
        }).unwrap();
    });
}

