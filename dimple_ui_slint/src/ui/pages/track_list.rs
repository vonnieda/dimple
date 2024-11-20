use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::TrackListAdapter;
use crate::ui::Navigator;
use crate::ui::Page;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
use slint::Model as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;

// https://github.com/slint-ui/slint/discussions/2329
pub fn track_list(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let tracks: Vec<Track> = app.librarian.list();
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
            let adapter = TrackListAdapter {
                rows: ModelRc::new(rows),
                keys: ModelRc::from(keys.as_slice()),
            };
            ui.set_track_list(adapter);
            ui.set_page(Page::TrackList);
        })
        .unwrap();
    });
}

pub fn track_list_track_selected(app: &App, row: i32) {
    app.ui.upgrade_in_event_loop(move |ui| {
        let adapter = ui.get_track_list();
        // let rows = adapter.rows.as_any().downcast_ref::<VecModel<ModelRc<StandardListViewItem>>>().unwrap();
        let key = adapter.keys.row_data(row as usize).unwrap().to_string();
        ui.global::<Navigator>().invoke_navigate(format!("dimple://track/{}", &key).into());
    }).unwrap();
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
