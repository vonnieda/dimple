use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
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
        ui.global::<TrackListAdapter>().on_sort_ascending(move |col| sort_ascending(&app, col));
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_sort_descending(move |col| sort_descending(&app, col));
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_add_to_queue(move |key| add_to_queue(&app, &key));
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_play_next(move |key| play_next(&app, &key));
    }).unwrap();
}

pub fn track_list(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let tracks: Vec<Track> = app.library.query("
            SELECT * FROM Track
            ORDER BY artist asc, album asc, media_position asc, title asc
        ", ());
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<TrackListAdapter>().set_row_data(row_data(&tracks));
            ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
            ui.set_page(Page::TrackList);
        })
        .unwrap();
    });
}

fn row_data(tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms as u64))
            .map(|dur| format_length(dur));
        row.push(track.title.unwrap_or_default().as_str().into()); // Title
        row.push(track.album.unwrap_or_default().as_str().into()); // Album
        row.push(track.artist.unwrap_or_default().as_str().into()); // Artist
        row.push(track.media_position.unwrap_or_default().to_string().as_str().into()); // Track #
        row.push(track.plays.to_string().as_str().into()); // Plays
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

fn sort_ascending(app: &App, col: i32) {
    let app = app.clone();
    thread::spawn(move || {
        let query = match col {
            0 => "SELECT * FROM Track ORDER BY title asc",
            1 => "SELECT * FROM Track ORDER BY album asc",
            2 => "SELECT * FROM Track ORDER BY artist asc",
            3 => "SELECT * FROM Track ORDER BY media_position asc",
            4 => "SELECT * FROM Track ORDER BY plays asc",
            5 => "SELECT * FROM Track ORDER BY length_ms asc",
            _ => "SELECT * FROM Track ORDER BY title asc",
        };
        let tracks: Vec<Track> = app.library.query(query, ());
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<TrackListAdapter>().set_row_data(row_data(&tracks));
            ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
        })
        .unwrap();
    });
}

fn sort_descending(app: &App, col: i32) {
    let app = app.clone();
    thread::spawn(move || {
        let query = match col {
            0 => "SELECT * FROM Track ORDER BY title desc",
            1 => "SELECT * FROM Track ORDER BY album desc",
            2 => "SELECT * FROM Track ORDER BY artist desc",
            3 => "SELECT * FROM Track ORDER BY media_position desc",
            4 => "SELECT * FROM Track ORDER BY plays desc",
            5 => "SELECT * FROM Track ORDER BY length_ms desc",
            _ => "SELECT * FROM Track ORDER BY title desc",
        };
        let tracks: Vec<Track> = app.library.query(query, ());
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<TrackListAdapter>().set_row_data(row_data(&tracks));
            ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
        })
        .unwrap();
    });
}

fn add_to_queue(app: &App, key: &str) {
    let app = app.clone();
    let key = key.to_string();
    app.ui.upgrade_in_event_loop(move |ui| {
        // let row_data = ui.global::<TrackListAdapter>().get_row_data();
        // let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
        // let key = cell_data.text.as_str();
        let play_queue = app.player.queue();
        app.library.playlist_add(&play_queue, &key);
    }).unwrap();
}

fn play_now(app: &App, key: &str) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        // TODO think about ephemeral or secondary playlist, or even
        // a playlist inserted inbetween the playing items
        // let row_data = ui.global::<TrackListAdapter>().get_row_data();
        // let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
        // let key = cell_data.text.as_str();
        // let play_queue = app.player.queue();
        // app.library.playlist_insert(&play_queue, &key, row);
    }).unwrap();
}

fn play_next(app: &App, key: &str) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        // TODO think about ephemeral or secondary playlist, or even
        // a playlist inserted inbetween the playing items
        // let row_data = ui.global::<TrackListAdapter>().get_row_data();
        // let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
        // let key = cell_data.text.as_str();
        // let play_queue = app.player.queue();
        // app.library.playlist_insert(&play_queue, &key);
    }).unwrap();
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
