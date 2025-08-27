use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use crate::ui::app_window_controller::App;
use dimple_core::library::Library;
use dimple_core::model::Track;
use dimple_db::db::query::QuerySubscription;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use crate::ui::TrackListAdapter;

pub struct TrackListController {
    _tracks_subscription: QuerySubscription,
}

impl TrackListController {
    pub fn new(app: &App) -> Result<Self> {
        let sql = "
            SELECT DISTINCT Track.* 
            FROM Track
            JOIN TrackSource ON TrackSource.track_id = Track.id 
            JOIN MediaFile ON MediaFile.id = TrackSource.media_file_id 
            WHERE content IS NOT NULL 
                OR Track.save = true 
            ORDER BY Track.title ASC

        ";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let tracks_subscription = app.library.db.query_subscribe(
            sql,
            (),
            move |tracks: Vec<Track>| {
                let library = library.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    ui.global::<TrackListAdapter>().set_row_data(row_data(&library, &tracks));
                    ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
                }).unwrap();
            },
        )?;

        // TODO
        // let app_ = app.clone();
        // app.ui.upgrade_in_event_loop(move |ui| {
        //     let app = app_;
        //     ui.global::<TrackListAdapter>().on_sort_table(move |col, ascending| {
        //         sort_table(&app, &query_arc, col, ascending)
        //     });
        // }).unwrap();

        Ok(Self {
            _tracks_subscription: tracks_subscription,
        })
    }
}

fn row_data(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms))
            .map(format_length);
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
        .map(|track| track.id.clone().unwrap())
        .map(SharedString::from)
        .collect();
    keys.as_slice().into()
}

fn sort_table(app: &App, current_query: &Arc<Mutex<String>>, col: i32, ascending: bool) {
    let columns = ["title", "album", "artist", "position", "plays", "length_ms"];
    let query = format!("SELECT * FROM Track ORDER BY {} {}", 
        columns[col as usize], 
        if ascending { "asc" } else { "desc" });
    
    // Update the query for the subscription (this will trigger a refresh)
    *current_query.lock().unwrap() = query.clone();
    
    // For now, do immediate update - TODO: make subscription dynamic
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
    format!("{minutes}:{seconds:02}")
}
