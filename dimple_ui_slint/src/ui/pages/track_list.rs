use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::ModelBasics;
use dimple_core::model::Track;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use crate::ui::TrackListAdapter;

pub fn track_list_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_sort_table(move |col, ascending| sort_table(&app, col, ascending));
    }).unwrap();
}

pub fn track_list(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let tracks = Track::list(&app.library);
        let library = app.library.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<TrackListAdapter>().set_row_data(row_data(&library, &tracks));
            ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
            ui.set_page(Page::TrackList);
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
        row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
        row.push(track.album_name(library).unwrap_or_default().as_str().into()); // Album
        row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
        row.push(track.position.unwrap_or_default().to_string().as_str().into()); // Track #
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

fn sort_table(app: &App, col: i32, ascending: bool) {
    let columns = vec!["title", "album", "artist", "position", "plays", "length_ms"];
    let query = format!("SELECT * FROM Track ORDER BY {} {}", 
        columns[col as usize], 
        if ascending { "asc" } else { "desc" });
    let tracks: Vec<Track> = app.library.query(&query, ());
    let library = app.library.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<TrackListAdapter>().set_row_data(row_data(&library, &tracks));
        ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
    })
    .unwrap();
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
