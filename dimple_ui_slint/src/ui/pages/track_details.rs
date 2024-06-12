use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
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
use crate::ui::TrackDetailsAdapter;
use dimple_core::db::Db;
use crate::ui::CardAdapter;
use crate::ui::LinkAdapter;

pub fn track_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let track: Track = librarian.get(&Track {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        let mut genres: Vec<Genre> = librarian
            .list(&Genre::default().into(), &Some(track.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        genres.sort_by_key(|genre| genre.name.clone().unwrap_or_default().to_lowercase());

        // let mut release_groups: Vec<_> = librarian
        //     .list(&ReleaseGroup::default().into(), &Some(artist.model()))
        //     .unwrap()
        //     .map(ReleaseGroup::from)
        //     // .filter(|r| r.secondary_types.is_empty())
        //     .collect();
        // release_groups.sort_by_key(|r| r.first_release_date.to_owned());
        // release_groups.reverse();

        // let releases = release_groups;

        ui.upgrade_in_event_loop(move |ui| {
            // // TODO I hate all this duplication, but each one needs to filter on
            // // a different string, and then the closure needs to access a different
            // // field. So, duplication.
            // // TODO need to switch primary type to an enum
            // let albums: Vec<CardAdapter> = releases.iter().cloned()
            //     // TODO add the to_lower to others for now, then replace with enum.
            //     .filter(|release| release.primary_type.clone().map(|s| s.to_lowercase()) == Some("album".to_string()))
            //     .enumerate()
            //     .map(|(index, release)| {
            //         let mut card: CardAdapter = release.clone().into();
            //         card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //             let mut card = ui.get_artist_details().albums.row_data(index).unwrap();
            //             card.image.image = image;
            //             ui.get_artist_details().albums.set_row_data(index, card);
            //         });
            //         card
            //     })
            //     .collect();
            // let eps: Vec<CardAdapter> = releases.iter().cloned()
            //     .filter(|release| release.primary_type == Some("Ep".to_string()))
            //     .enumerate()
            //     .map(|(index, release)| {
            //         let mut card: CardAdapter = release.clone().into();
            //         card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //             let mut card = ui.get_artist_details().eps.row_data(index).unwrap();
            //             card.image.image = image;
            //             ui.get_artist_details().eps.set_row_data(index, card);
            //         });
            //         card
            //     })
            //     .collect();
            // let singles: Vec<CardAdapter> = releases.iter().cloned()
            //     .filter(|release| release.primary_type == Some("Single".to_string()))
            //     .enumerate()
            //     .map(|(index, release)| {
            //         let mut card: CardAdapter = release.clone().into();
            //         card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //             let mut card = ui.get_artist_details().singles.row_data(index).unwrap();
            //             card.image.image = image;
            //             ui.get_artist_details().singles.set_row_data(index, card);
            //         });
            //         card
            //     })
            //     .collect();
            // let others: Vec<CardAdapter> = releases.iter().cloned()
            //     .filter(|release| {
            //         let pt = release.primary_type.clone();
            //         pt != Some("Album".to_string()) && pt != Some("Ep".to_string()) && pt != Some("Single".to_string())
            //     })
            //     .enumerate()
            //     .map(|(index, release)| {
            //         let mut card: CardAdapter = release.clone().into();
            //         card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //             let mut card = ui.get_artist_details().others.row_data(index).unwrap();
            //             card.image.image = image;
            //             ui.get_artist_details().others.set_row_data(index, card);
            //         });
            //         card
            //     })
            //     .collect();

            let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
                LinkAdapter {
                    name: genre.name.unwrap().into(),
                    url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
                }
            }).collect();

            let links: Vec<LinkAdapter> = track.recording.links.iter().map(|link| {
                LinkAdapter {
                    name: link.into(),
                    url: link.into(),
                }
            }).collect();

            let mut adapter = TrackDetailsAdapter {
                card: track.clone().into(),
                // disambiguation: track.disambiguation.clone().unwrap_or_default().into(),
                summary: track.recording.summary.clone().unwrap_or_default().into(),
                // albums: ModelRc::from(albums.as_slice()),
                // singles: ModelRc::from(singles.as_slice()),
                // eps: ModelRc::from(eps.as_slice()),
                // others: ModelRc::from(others.as_slice()),
                genres: ModelRc::from(genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                dump: serde_json::to_string_pretty(&track).unwrap().into(),
                ..Default::default()
            };

            adapter.card.image.image = images.lazy_get(track.model(), 275, 275, |ui, image| {
                let mut model = ui.get_track_details();
                model.card.image.image = image;
                ui.set_track_details(model);
            });

            ui.set_track_details(adapter);
            ui.set_page(Page::TrackDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

