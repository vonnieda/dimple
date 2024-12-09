use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
use dimple_core::model::Track;
use slint::Model;
use slint::Model as _;
use slint::ModelExt as _;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use crate::ui::TrackListAdapter;

pub fn track_list_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_current_row_changed(move |row| row_selected(&app, row));
        ui.global::<TrackListAdapter>().on_sort_model(sort_model);
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_add_to_queue(move |row| add_to_queue(&app, row));
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_play_now(move |row| play_now(&app, row));
        let app = app_.clone();
        ui.global::<TrackListAdapter>().on_play_next(move |row| play_next(&app, row));
    }).unwrap();
}

pub fn track_list(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let tracks: Vec<Track> = app.library.list();
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<TrackListAdapter>().set_row_data(row_data(&tracks));
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
        let length = Duration::from_millis(track.length_ms.unwrap_or_default() as u64);
        let length = format_length(length);
        row.push(StandardListViewItem::from(track.title.unwrap_or_default().as_str())); // Title
        row.push(StandardListViewItem::from(track.album.unwrap_or_default().as_str())); // Album
        row.push(StandardListViewItem::from(track.artist.unwrap_or_default().as_str())); // Artist
        row.push(StandardListViewItem::from("1")); // Track #
        row.push(track.plays.to_string().as_str().into()); // Plays
        row.push(StandardListViewItem::from(length.as_str())); // Length
        row.push(StandardListViewItem::from(track.key.unwrap().as_str())); // Key (Hidden)
        row_data.push(row.into());
    }
    row_data.into()
}

fn row_selected(app: &App, row: i32) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |_ui| {
        println!("row_selected {}", row);
        // let adapter = ui.get_track_list();
        // let rows = adapter.rows.as_any().downcast_ref::<VecModel<ModelRc<StandardListViewItem>>>().unwrap();
        // let key = adapter.keys.row_data(row as usize).unwrap().to_string();
        // ui.global::<Navigator>().invoke_navigate(format!("dimple://track/{}", &key).into());
        // let play_queue = app.player.queue();
        // app.library.playlist_add(&play_queue, &key);
    }).unwrap();
}

fn add_to_queue(app: &App, row: i32) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let row_data = ui.global::<TrackListAdapter>().get_row_data();
        let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
        let key = cell_data.text.as_str();
        let play_queue = app.player.queue();
        app.library.playlist_add(&play_queue, &key);
    }).unwrap();
}

fn play_now(app: &App, row: i32) {
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

fn play_next(app: &App, row: i32) {
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
