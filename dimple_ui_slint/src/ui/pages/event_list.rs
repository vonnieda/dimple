use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::EventListAdapter;
use crate::ui::Navigator;
use crate::ui::Page;
use dimple_core::model::Event;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
use slint::Model as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;

pub fn event_list(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let events: Vec<Event> = app.library.query("SELECT * FROM Event ORDER BY timestamp DESC", ());
        app.ui.upgrade_in_event_loop(move |ui| {
            let rows: VecModel<ModelRc<StandardListViewItem>> = Default::default();

            // { title: "Date" },
            // { title: "Type" },
            // { title: "Artist" },
            // { title: "Album" },
            // { title: "Title" },
            
            for event in &events {
                let event = event.clone();
                let row = Rc::new(VecModel::default());
                row.push(StandardListViewItem::from(event.timestamp.as_str())); // Date
                row.push(StandardListViewItem::from(event.event_type.as_str())); // Type
                row.push(StandardListViewItem::from(event.artist.unwrap_or_default().as_str())); // Artist
                row.push(StandardListViewItem::from(event.album.unwrap_or_default().as_str())); // Album
                row.push(StandardListViewItem::from(event.title.unwrap_or_default().as_str())); // Title
                rows.push(row.into());
            }
            let keys: Vec<_> = events.iter()
                .map(|event| event.key.clone().unwrap())
                .map(|key| SharedString::from(key))
                .collect();
            let adapter = EventListAdapter {
                rows: ModelRc::new(rows),
                keys: ModelRc::from(keys.as_slice()),
            };
            ui.set_event_list(adapter);
            ui.set_page(Page::EventList);
        })
        .unwrap();
    });
}

pub fn event_list_event_selected(app: &App, row: i32) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let adapter = ui.get_track_list();
        // let rows = adapter.rows.as_any().downcast_ref::<VecModel<ModelRc<StandardListViewItem>>>().unwrap();
        let key = adapter.keys.row_data(row as usize).unwrap().to_string();
        // ui.global::<Navigator>().invoke_navigate(format!("dimple://track/{}", &key).into());
        // let play_queue = app.player.queue();
        // app.library.playlist_add(&play_queue, &key);
    }).unwrap();
}

