use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Medium;
use dimple_core::model::Model;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use dimple_core::db::Db;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;

pub fn search(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();

    /// So this will be the first new model controller with the intention of
    /// being reactive. The goal will be to get an iterator from the search
    /// and feed those objects in realtime over to the UI as they come in.
    /// This will require adding a sort model, and figuring out that stuff
    /// so that the results stay sorted in the UI.

    std::thread::spawn(move || {
        log::info!("{}", url);
        let url = Url::parse(&url).unwrap();
        let query = url.path_segments().unwrap().next().unwrap();
        // TODO wtf? really?
        let query = percent_encoding::percent_decode_str(query).decode_utf8_lossy().to_string();

        let results: Vec<Model> = librarian
            .search(&query)
            .unwrap()
            .collect();

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = results.iter().cloned().enumerate()
                .map(|(index, result)| {
                    let mut card: CardAdapter = model_card(&result);
                    card.image.image = images.lazy_get(result, 200, 200, move |ui, image| {
                        let mut card = ui.get_search().cards.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_search().cards.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_search(adapter);
            ui.set_page(Page::Search);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}


fn model_card(model: &Model) -> CardAdapter {
    match model {
        Model::Artist(artist) => artist.clone().into(),
        Model::ReleaseGroup(release_group) => release_group.clone().into(),
        Model::Release(release) => release.clone().into(),
        Model::Genre(genre) => genre.clone().into(),
        _ => todo!(),
    }
}
