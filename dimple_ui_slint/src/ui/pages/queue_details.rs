use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::Page;
use dimple_core::model::Playlist;
use dimple_core::model::Track;
use slint::Model as _;
use slint::ModelExt as _;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use crate::ui::QueueDetailsAdapter;
use crate::ui::Navigator;

pub fn queue_details_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<QueueDetailsAdapter>().on_sort_model(sort_model);

        let app = app_.clone();
        ui.global::<QueueDetailsAdapter>().on_play_now(move |row| {
            app.player.set_queue_index(row as usize);
            app.player.play();
        });

        ui.global::<QueueDetailsAdapter>().on_remove_row(move |_row| {
            todo!()
        });

        let app = app_.clone();
        ui.global::<QueueDetailsAdapter>().on_remove_all(move || {
            let queue = app.player.queue();
            app.library.playlist_clear(&queue);
            app.ui.upgrade_in_event_loop(|ui| ui.global::<Navigator>().invoke_navigate("dimple://refresh".into()));
        });
    }).unwrap();
}

pub fn queue_details(url: &str, app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let playlist: Playlist = app.player.queue();
        let tracks = playlist.tracks(&app.library);
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<QueueDetailsAdapter>().set_row_data(row_data(&tracks));
            ui.set_page(Page::QueueDetails);
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
