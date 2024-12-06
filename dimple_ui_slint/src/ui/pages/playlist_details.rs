use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
use dimple_core::model::Model;
use dimple_core::model::Playlist;
use dimple_core::model::Track;
use slint::Model as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use url::Url;
use crate::ui::PlaylistDetailsAdapter;

// https://github.com/slint-ui/slint/discussions/2329
pub fn playlist_details(url: &str, app: &App) {
    let app = app.clone();
    let url = url.to_owned();
    let library = app.library.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    thread::spawn(move || {
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();
        let playlist: Playlist = library.get(key).unwrap();
        let tracks = playlist.tracks(&library);
        app.ui.upgrade_in_event_loop(move |ui| {
            let rows: VecModel<ModelRc<StandardListViewItem>> = Default::default();
            for track in &tracks {
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
                rows.push(row.into());
            }
            let keys: Vec<_> = tracks.iter()
                .map(|track| track.key.clone().unwrap())
                .map(|key| SharedString::from(key))
                .collect();
            let adapter = PlaylistDetailsAdapter {
                rows: ModelRc::new(rows),
                keys: ModelRc::from(keys.as_slice()),
            };
            ui.set_playlist_details(adapter);
            ui.set_page(Page::PlaylistDetails);
        })
        .unwrap();
    });
}

pub fn playlist_details_track_selected(app: &App, row: i32) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let adapter = ui.get_track_list();
        // let rows = adapter.rows.as_any().downcast_ref::<VecModel<ModelRc<StandardListViewItem>>>().unwrap();
        // let key = adapter.keys.row_data(row as usize).unwrap().to_string();
        // ui.global::<Navigator>().invoke_navigate(format!("dimple://track/{}", &key).into());
        // let play_queue = app.player.queue();
        // TODO this only belongs on queue details, not playlist details
        app.player.set_queue_index(row as usize);
    }).unwrap();
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
