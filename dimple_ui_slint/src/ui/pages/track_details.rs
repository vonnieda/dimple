use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Track;
use slint::ComponentHandle;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
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

        ui.upgrade_in_event_loop(move |ui| {
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
                summary: track.recording.summary.clone().unwrap_or_default().into(),
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

