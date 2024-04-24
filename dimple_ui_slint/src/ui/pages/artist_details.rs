use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Model;
use dimple_core::model::ReleaseGroup;
use slint::ComponentHandle;
use slint::Image;
use slint::Model as _;
use slint::ModelExt;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::ArtistDetailsAdapter;
use dimple_core::db::Db;
use crate::ui::CardAdapter;

pub fn artist_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let artist: Artist = librarian.get(&Artist {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        let mut release_groups: Vec<ReleaseGroup> = librarian
            .list(&ReleaseGroup::default().into(), Some(&Model::Artist(artist.clone())))
            .unwrap()
            .map(Into::into)
            .collect();
        release_groups.sort_by_key(|f| f.first_release_date.to_owned());
        release_groups.reverse();

        ui.upgrade_in_event_loop(move |ui| {
            let release_groups: Vec<CardAdapter> = release_groups.iter().cloned().enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_artist_details().albums.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_artist_details().albums.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let release_groups = ModelRc::from(release_groups.as_slice());

            // let albums = release_groups.filter(|card| {
            //     true
            // });

            let mut adapter = ArtistDetailsAdapter {
                card: artist.clone().into(),
                disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
                summary: artist.summary.clone().unwrap_or_default().into(),
                albums: ModelRc::new(release_groups.clone().filter(|card| true)),
                // singles: ModelRc::from(singles.as_slice()),
                // eps: ModelRc::from(eps.as_slice()),
                others: release_groups,
                // genres: link_adapters(genres),
                // links: link_adapters(artist_links(&artist)),
                dump: serde_json::to_string_pretty(&artist).unwrap().into(),
                ..Default::default()
            };
            adapter.card.image.image = images.get(artist.model(), 275, 275);
            ui.set_artist_details(adapter);
            ui.set_page(Page::ArtistDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

