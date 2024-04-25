use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::TrackListAdapter;
use crate::ui::Page;
use dimple_core::model::Entity;
use dimple_core::model::Track;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;
use dimple_core::db::Db;

// https://github.com/slint-ui/slint/discussions/2329
pub fn track_list(app: &App) {
    let app = app.clone();
    let tracks: Vec<Track> = app.librarian
        .list(&Track::default().model(), None)
        .unwrap().map(Into::<Track>::into).collect();
    thread::spawn(move || {
        app.ui.upgrade_in_event_loop(move |ui| {
            let rows: VecModel<ModelRc<StandardListViewItem>> = Default::default();
            for track in tracks {
                let row = Rc::new(VecModel::default());
                // Title, Album, Artist, Track #, Plays, Length
                let length = Duration::from_secs(track.length.unwrap_or_default() as u64);
                let length = format_length(length);
                row.push(StandardListViewItem::from(track.title.unwrap_or_default().as_str()));
                row.push(StandardListViewItem::from("Unknown Album"));
                row.push(StandardListViewItem::from("Unknown Artist"));
                row.push(StandardListViewItem::from(format!("{}", track.position.unwrap_or_default()).as_str()));
                row.push(StandardListViewItem::from("1"));
                row.push(StandardListViewItem::from(length.as_str()));
                rows.push(row.into());
            }
            let adapter = TrackListAdapter {
                rows: ModelRc::new(rows),
            };
            ui.set_track_list(adapter);
            ui.set_page(Page::TrackList);
        })
        .unwrap();
    });
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
