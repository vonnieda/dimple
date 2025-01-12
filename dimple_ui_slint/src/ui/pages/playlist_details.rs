use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
use crate::ui::Navigator;
use dimple_core::library::Library;
use dimple_core::model::Playlist;
use dimple_core::model::Track;
use slint::Model as _;
use slint::ModelExt as _;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use url::Url;
use crate::ui::PlaylistDetailsAdapter;

/// - [x] list
/// - [x] detail
/// - [x] new
///     - [x] rename
/// - [/] delete
/// - [/] play
pub fn playlist_details_init(_app: &App) {
    let app = _app.clone();
    _app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<PlaylistDetailsAdapter>().on_sort_model(sort_model);
        {
            let app = app.clone();
            ui.global::<PlaylistDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        }
        // ui.global::<PlaylistDetailsAdapter>().on_add_to_queue(move |key| add_to_queue(&app, &key));
        // ui.global::<PlaylistDetailsAdapter>().on_set_download(move |key, checked| set_download(&app, &key, checked));
        // ui.global::<PlaylistDetailsAdapter>().on_set_love(move |key, checked| set_love(&app, &key, checked));
        {
            let app = app.clone();
            ui.global::<PlaylistDetailsAdapter>().on_delete(move |key| delete(&app, &key));
        }
        {
            let app = app.clone();
            ui.global::<PlaylistDetailsAdapter>().on_set_name(move |key, name| set_name(&app, &key, &name));
        }
    }).unwrap();
}

pub fn playlist_details(url: &str, app: &App) {
    let app = app.clone();
    let url = url.to_owned();
    thread::spawn(move || {
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap().to_string();
        let playlist: Playlist = app.library.get(&key).unwrap();
        let tracks = playlist.tracks(&app.library);
        let library = app.library.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<PlaylistDetailsAdapter>().set_key(key.into());
            ui.global::<PlaylistDetailsAdapter>().set_row_data(row_data(&library, &tracks));
            ui.global::<PlaylistDetailsAdapter>().set_name(playlist.name.clone()
                .unwrap_or("(Nameless Playlist)".to_string()).into());
            ui.set_page(Page::PlaylistDetails);
        })
        .unwrap();
    });
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
        row.push(track.album_name(library).unwrap_or_default().as_str().into()); // Album
        row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
        row.push(length.unwrap_or_default().as_str().into()); // Length
        row_data.push(row.into());
    }
    row_data.into()
}

fn set_name(app: &App, key: &str, name: &str) {
    let mut playlist = app.library.get::<Playlist>(key).unwrap();
    playlist.name = Some(name.to_string());
    app.library.save(&playlist);
    app.ui.upgrade_in_event_loop(move |ui| {
        // TODO should not be needed with library.on_change
        ui.global::<PlaylistDetailsAdapter>().set_name(playlist.name.clone()
            .unwrap_or("(Nameless Playlist)".to_string()).into());
    }).unwrap();
}

fn play_now(app: &App, key: &str) {
    let playlist = app.library.get::<Playlist>(key).unwrap();
    let play_queue = app.player.queue();
    for track in playlist.tracks(&app.library) {
        app.library.playlist_add(&play_queue, &track.key.unwrap());
    }
    app.navigate("dimple://queue".into());
}

fn delete(app: &App, key: &str) {
    // let mut playlist = app.library.get::<Playlist>(key).unwrap();
    // playlist.name = Some(name.to_string());
    // app.library.save(&playlist);
    // app.ui.upgrade_in_event_loop(move |ui| {
    //     // TODO should not be needed with library.on_change
    //     ui.global::<PlaylistDetailsAdapter>().set_name(playlist.name.clone()
    //         .unwrap_or("(Nameless Playlist)".to_string()).into());
    // }).unwrap();
}

fn sort_model(
    source_model: ModelRc<ModelRc<StandardListViewItem>>,
    sort_index: i32,
    sort_ascending: bool,
) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut model = source_model.clone();

    if sort_index >= 0 {
        model = Rc::new(model.clone().sort_by(move |r_a, r_b| {
            let c_a = r_a.row_data(sort_index as usize).unwrap();
            let c_b = r_b.row_data(sort_index as usize).unwrap();

            if sort_ascending {
                c_a.text.cmp(&c_b.text)
            } else {
                c_b.text.cmp(&c_a.text)
            }
        }))
        .into();
    }

    model
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
