use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::TrackListAdapter;
use crate::ui::Page;
use dimple_core::model::Track;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;

// https://github.com/slint-ui/slint/discussions/2329
pub fn track_list(app: &App) {
    let app = app.clone();
    let tracks: Vec<Track> = app.librarian.list();
    thread::spawn(move || {
        app.ui.upgrade_in_event_loop(move |ui| {
            let rows: VecModel<ModelRc<StandardListViewItem>> = Default::default();
            for track in tracks {
                let row = Rc::new(VecModel::default());
                // Title, Album, Artist, Track #, Plays, Length
                let length = Duration::from_millis(track.length_ms.unwrap_or_default() as u64);
                let length = format_length(length);
                row.push(StandardListViewItem::from(track.title.unwrap_or_default().as_str()));
                row.push(StandardListViewItem::from(track.album.unwrap_or_default().as_str()));
                row.push(StandardListViewItem::from(track.artist.unwrap_or_default().as_str()));
                row.push(StandardListViewItem::from(format!("{}", track.media_position.unwrap_or_default()).as_str()));
                row.push(StandardListViewItem::from(length.as_str())); // len
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
