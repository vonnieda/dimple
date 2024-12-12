use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
use crate::ui::Navigator;
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

pub fn playlist_details_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app_;
        ui.global::<PlaylistDetailsAdapter>().on_current_row_changed(move |row| row_selected(&app, row));
        ui.global::<PlaylistDetailsAdapter>().on_sort_model(sort_model);
    }).unwrap();
}

pub fn playlist_details(url: &str, app: &App) {
    let app = app.clone();
    let url = url.to_owned();
    thread::spawn(move || {
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();
        let playlist: Playlist = app.library.get(key).unwrap();
        let tracks = playlist.tracks(&app.library);
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<PlaylistDetailsAdapter>().set_row_data(row_data(&tracks));
            ui.global::<PlaylistDetailsAdapter>().set_name(playlist.name.clone()
                .unwrap_or("(Nameless Playlist)".to_string()).into());
            ui.set_page(Page::PlaylistDetails);
        })
        .unwrap();
    });
}

fn row_data(tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for (i, track) in tracks.iter().enumerate() {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = Duration::from_millis(track.length_ms.unwrap_or_default() as u64);
        let length = format_length(length);
        row.push(i.to_string().as_str().into()); // # (Ordinal)
        row.push(StandardListViewItem::from(track.title.unwrap_or_default().as_str())); // Title
        row.push(StandardListViewItem::from(track.album.unwrap_or_default().as_str())); // Album
        row.push(StandardListViewItem::from(track.artist.unwrap_or_default().as_str())); // Artist
        row.push(StandardListViewItem::from(length.as_str())); // Length
        row.push(StandardListViewItem::from(track.key.unwrap().as_str())); // Key (Hidden)
        row_data.push(row.into());
    }
    row_data.into()
}

fn row_selected(app: &App, row: i32) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let row_data = ui.global::<PlaylistDetailsAdapter>().get_row_data();
        let cell_data = row_data.row_data(row as usize).unwrap().row_data(5).unwrap();
        let key = cell_data.text.as_str();
        ui.global::<Navigator>().invoke_navigate(format!("dimple://track/{}", &key).into());
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
