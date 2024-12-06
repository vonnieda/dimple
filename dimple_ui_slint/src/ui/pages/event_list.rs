use std::rc::Rc;
use std::thread;

use crate::ui::app_window_controller::App;
use crate::ui::EventListAdapter;
use crate::ui::Page;
use dimple_core::model::Event;
use slint::Model as _;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::ComponentHandle as _;
use slint::ModelExt as _;

pub fn event_list_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app_;
        ui.global::<EventListAdapter>().on_current_row_changed(move |row| row_selected(&app, row));
        ui.global::<EventListAdapter>().on_sort_model(sort_model);
    }).unwrap();
}

pub fn event_list(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        let events: Vec<Event> = app.library.query("SELECT * FROM Event ORDER BY timestamp DESC", ());
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<EventListAdapter>().set_row_data(row_data(&events));
            ui.set_page(Page::EventList);
        })
        .unwrap();
    });
}

fn row_data(events: &[Event]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for event in events {
        let event = event.clone();
        let row = Rc::new(VecModel::default());
        row.push(StandardListViewItem::from(event.timestamp.as_str())); // Date
        row.push(StandardListViewItem::from(event.event_type.as_str())); // Type
        row.push(StandardListViewItem::from(event.artist.unwrap_or_default().as_str())); // Artist
        row.push(StandardListViewItem::from(event.album.unwrap_or_default().as_str())); // Album
        row.push(StandardListViewItem::from(event.title.unwrap_or_default().as_str())); // Title
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

