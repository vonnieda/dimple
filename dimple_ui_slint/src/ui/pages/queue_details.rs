use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::Track;
use dimple_core::player::PlayerEvent;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use crate::ui::QueueDetailsAdapter;
use dimple_db::db::query::QuerySubscription;
use anyhow::Result;

pub struct QueueDetailsController {
    playlist_id: MutableStringParam,
    tracks_subscription: QuerySubscription,
}

impl QueueDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let playlist_id = MutableStringParam::new();
        
        // Get the current queue ID and set it
        let queue = app.player.queue();
        if let Some(id) = queue.id {
            playlist_id.set(&id);
        }
        
        // Set up tracks subscription
        let sql = "
            SELECT Track.*
            FROM PlaylistItem
            JOIN Track ON (Track.id = PlaylistItem.Track_id)
            WHERE PlaylistItem.playlist_id = ?1
            AND PlaylistItem.deleted = FALSE
            ORDER BY PlaylistItem.ordinal ASC, PlaylistItem.rowid ASC
        ";
        
        let library = app.library.clone();
        let ui = app.ui.clone();
        let player = app.player.clone();
        let tracks_subscription = app.library.db.query_subscribe(sql, (playlist_id.clone(),), move |tracks: Vec<Track>| {
            let library = library.clone();
            let player = player.clone();
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<QueueDetailsAdapter>().set_row_data(row_data(&library, &tracks));
                ui.global::<QueueDetailsAdapter>().set_row_keys(row_keys(&tracks));
                ui.global::<QueueDetailsAdapter>().set_current_row(player.current_queue_index() as i32);
            }).unwrap();
        })?;

        Self::queue_details_init(app);

        Ok(Self {
            playlist_id,
            tracks_subscription,
        })
    }

    fn queue_details_init(app: &App) {
        let app_ = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_.clone();
            ui.global::<QueueDetailsAdapter>().on_play_now(move |row| {
                app.player.set_queue_index(row as usize);
                app.player.play();
            });
            let app = app_.clone();
            ui.global::<QueueDetailsAdapter>().on_remove_row(move |row| {
                let queue = app.player.queue();
                queue.remove(&app.library, row as usize);
            });
            let app = app_.clone();
            ui.global::<QueueDetailsAdapter>().on_remove_all(move || {
                app.player.queue().clear(&app.library);
            });
        }).unwrap();

        let app1 = app.clone();
        app.player.notifier.observe(move |event| {
            match event {
                PlayerEvent::QueueIndex(index) => {
                    app1.ui.upgrade_in_event_loop(move |ui| {
                        ui.global::<QueueDetailsAdapter>().set_current_row(index as i32);
                    }).unwrap();
                },
                _ => (),
            }
        });
    }

    pub fn show(app: &App) {
        // Set the queue in the controller which will handle all subscriptions
        // Navigate to the queue details page
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.set_page(Page::QueueDetails);
        }).unwrap();
    }
}

fn row_data(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for (i, track) in tracks.iter().enumerate() {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms as u64))
            .map(|dur| format_length(dur));
        row.push((i + 1).to_string().as_str().into()); // # (Ordinal)
        row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
        row.push(track.album_name(library).unwrap_or_default().as_str().into()); // Album
        row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
        row.push(length.unwrap_or_default().as_str().into()); // Length
        row_data.push(row.into());
    }
    row_data.into()
}

fn row_keys(tracks: &[Track]) -> ModelRc<SharedString> {
    let keys: Vec<_> = tracks.iter()
        .map(|track| track.id.clone().unwrap())
        .map(|key| SharedString::from(key))
        .collect();
    keys.as_slice().into()
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
