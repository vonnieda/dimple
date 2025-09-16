use anyhow::Result;
use slint::ComponentHandle as _;
use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::GenreListAdapter;
use dimple_core::model::Genre;
use dimple_db::db::query::QuerySubscription;
use slint::ModelRc;

pub struct GenreListController {
    _genres_subscription: QuerySubscription,
}

impl GenreListController {
    pub fn new(app: &App) -> Result<Self> {
        let ui = app.ui.clone();
        let genres_subscription = app.library.db.query_subscribe("
            SELECT * 
            FROM Genre 
            WHERE save = TRUE 
            ORDER BY name ASC, disambiguation ASC
        ", (), move |genres: Vec<Genre>| {
                ui.upgrade_in_event_loop(move |ui| {
                    let cards = genre_cards(&genres);
                    let adapter = ui.global::<GenreListAdapter>();
                    adapter.set_cards(ModelRc::from(cards.as_slice()));
                }).unwrap();
            },
        )?;

        Ok(Self {
            _genres_subscription: genres_subscription,
        })
    }
}

fn genre_cards(genres: &[Genre]) -> Vec<CardAdapter> {
    genres.iter().cloned().map(Into::into).collect()
}
