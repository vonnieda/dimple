use dimple_core::model::Artist;
use dimple_core::model::Model;
use dimple_core::model::ReleaseGroup;
use dimple_librarian::librarian::Librarian;
use slint::ComponentHandle;
use slint::Image;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::images::get_model_image;
use crate::ui::images::lazy_load_images;
use crate::ui::AppWindow;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::ArtistDetailsAdapter;
use dimple_core::db::Db;
use crate::ui::CardAdapter;

pub fn artist_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    std::thread::spawn(move || {        
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Navigator>().set_busy(true);
        }).unwrap();

        let url = Url::parse(&url).unwrap();
        let key = url.path_segments()
            .ok_or("missing path").unwrap()
            .nth(0)
            .ok_or("missing key").unwrap();

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

        let albums: Vec<ReleaseGroup> = release_groups.iter()
            .filter(|rg| rg.primary_type == Some("album".to_string()))
            .cloned()
            .collect();
        let singles: Vec<ReleaseGroup> = release_groups.iter()
            .filter(|rg| rg.primary_type == Some("single".to_string()))
            .cloned()
            .collect();
        let eps: Vec<ReleaseGroup> = release_groups.iter()
            .filter(|rg| rg.primary_type == Some("ep".to_string()))
            .cloned()
            .collect();
        let others: Vec<ReleaseGroup> = release_groups.iter()
            .filter(|rg| rg.primary_type != Some("album".to_string()) 
                && rg.primary_type != Some("single".to_string()) 
                && rg.primary_type != Some("ep".to_string()))
            .cloned()
            .collect();

        ui.upgrade_in_event_loop(move |ui| {
            let albums: Vec<CardAdapter> = albums.iter().cloned().map(Into::into).collect();
            let singles: Vec<CardAdapter> = singles.iter().cloned().map(Into::into).collect();
            let eps: Vec<CardAdapter> = eps.iter().cloned().map(Into::into).collect();
            let others: Vec<CardAdapter> = others.iter().cloned().map(Into::into).collect();

            let mut adapter = ArtistDetailsAdapter {
                // TODO need to load the images for the card and for all the
                // generated cards.
                card: artist.clone().into(),
                disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
                summary: artist.summary.clone().unwrap_or_default().into(),
                albums: ModelRc::from(albums.as_slice()),
                singles: ModelRc::from(singles.as_slice()),
                eps: ModelRc::from(eps.as_slice()),
                others: ModelRc::from(others.as_slice()),
                // genres: link_adapters(genres),
                // links: link_adapters(artist_links(&artist)),
                dump: serde_json::to_string_pretty(&artist).unwrap().into(),
                ..Default::default()
            };
            adapter.card.image.image = Image::from_rgba8_premultiplied(
                get_model_image(&librarian, &artist.clone().into(), 275, 275));
            ui.set_artist_details(adapter);
            ui.set_page(Page::ArtistDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

