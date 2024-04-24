use crate::ui::app_window_controller::App;
use crate::ui::images::lazy_load_images;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Artist;
use slint::ComponentHandle;
use slint::Image;
use slint::Model;
use slint::ModelRc;

pub fn artist_list(app: &App) {
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();

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

            // TODO hate this.
            let models: Vec<dimple_core::model::Model> = artists.iter().cloned().map(Into::into).collect();
            lazy_load_images(&librarian, models.as_slice(), ui.as_weak(), |ui| ui.get_artist_list().cards);

            // TODO replace with something like this.
            // let artist = Artist::default();
            // images.get2(artist.models(), 200, 200, |ui, image| {
            //     ui.get_artist_details().card.image.image = image;
            // });
        }).unwrap();
    });
}

fn set_artist_list_image(ui: AppWindow, index: usize, image: Image) {
    let mut card = ui.get_artist_list().cards.row_data(index).unwrap();
    card.image.image = image;
    ui.get_artist_list().cards.set_row_data(index, card);
}
