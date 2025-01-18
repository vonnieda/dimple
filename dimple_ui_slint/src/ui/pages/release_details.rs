use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseDetailsAdapter;

pub fn release_details_init(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app1.clone();
        ui.global::<ReleaseDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = app1.clone();
        ui.global::<ReleaseDetailsAdapter>().on_play_next(move |key| play_next(&app, &key));
        let app = app1.clone();
        ui.global::<ReleaseDetailsAdapter>().on_play_later(move |key| play_later(&app, &key));
        let app = app1.clone();
        ui.global::<ReleaseDetailsAdapter>().on_play_track_now(move |key| play_track_now(&app, &key));
        let app = app1.clone();
        ui.global::<ReleaseDetailsAdapter>().on_play_track_next(move |key| play_track_next(&app, &key));
        let app = app1.clone();
        ui.global::<ReleaseDetailsAdapter>().on_play_track_later(move |key| play_track_later(&app, &key));
    }).unwrap();

    
    // TODO filter events by key - but we can't get the key without the
    // UI, so rethink the whole mess.
    let app1 = app.clone();
    app.library.on_change(Box::new(move |event| if event.type_name == "Release" { update_model(&app1) }));
}

pub fn release_details(url: &str, app: &App) {
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();

    let app1 = app.clone();
    let key1 = key.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<ReleaseDetailsAdapter>().set_key(key1.clone().into());
        update_model(&app1);
        ui.set_page(Page::ReleaseDetails);

        let app2 = app1.clone();
        let key2 = key1.clone();
        std::thread::spawn(move || {
            if let Some(release) = Release::get(&app2.library, &key2) {
                librarian::refresh_metadata(&app2.library, &app2.plugins, &release);
            }
        });            
    }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<ReleaseDetailsAdapter>().get_key();
        if key.is_empty() {
            return
        }
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let release = Release::get(&library, &key).unwrap();
            let artists = release.artists(&library);
            let genres = release.genres(&library);
            let links = release.links(&library);
            let tracks = release.tracks(&app.library);
            let ui = app.ui.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut card: CardAdapter = release.clone().into();                
                card.image.image = app.images.lazy_get(release.clone(), 275, 275, |ui, image| {
                    let mut card = ui.global::<ReleaseDetailsAdapter>().get_card();
                    card.image.image = image;
                    ui.global::<ReleaseDetailsAdapter>().set_card(card);
                });

                let artists = artist_links(&artists);
                let genres = genre_links(&genres);
                let links = link_links(&links);
    
                ui.global::<ReleaseDetailsAdapter>().set_card(card.into());
                ui.global::<ReleaseDetailsAdapter>().set_key(release.key.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_release_type(release.release_group_type.clone().unwrap_or("Release".to_string()).into());
                ui.global::<ReleaseDetailsAdapter>().set_artists(ModelRc::from(artists.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_dump(format!("{:?}", release).into());
                ui.global::<ReleaseDetailsAdapter>().set_row_data(row_data(&library, &tracks));
                ui.global::<ReleaseDetailsAdapter>().set_row_keys(row_keys(&tracks));
            }).unwrap();
        });
    }).unwrap();
}

fn play_now(app: &App, key: &str) {
    app.player.play_now(&Release::get(&app.library, key).unwrap());
}

fn play_next(app: &App, key: &str) {
    app.player.play_next(&Release::get(&app.library, key).unwrap());
}

fn play_later(app: &App, key: &str) {
    app.player.play_later(&Release::get(&app.library, key).unwrap());
}

fn play_track_now(app: &App, key: &str) {
    app.player.play_now(&Track::get(&app.library, key).unwrap());
}

fn play_track_next(app: &App, key: &str) {
    app.player.play_next(&Track::get(&app.library, key).unwrap());
}

fn play_track_later(app: &App, key: &str) {
    app.player.play_later(&Track::get(&app.library, key).unwrap());
}

fn row_data(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms as u64))
            .map(|dur| format_length(dur));
        row.push(track.position.unwrap_or_default().to_string().as_str().into()); // Track #
        row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
        row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
        row.push(length.unwrap_or_default().as_str().into()); // Length
        row_data.push(row.into());
    }
    row_data.into()
}

fn row_keys(tracks: &[Track]) -> ModelRc<SharedString> {
    let keys: Vec<_> = tracks.iter()
        .map(|track| track.key.clone().unwrap())
        .map(|key| SharedString::from(key))
        .collect();
    keys.as_slice().into()
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn artist_links(artists: &[Artist]) -> Vec<LinkAdapter> {
    artists.iter().map(|artist| {
        LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn link_links(links: &[Link]) -> Vec<LinkAdapter> {
    links.iter().map(|link| {
        LinkAdapter {
            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
            url: link.url.clone().into(),
        }
    }).collect()
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}

