use std::rc::Rc;
use std::time::Duration;

use dimple_core::library::Library;
use dimple_core::model::Track;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::TrackListAdapter;
use crate::ui::LinkAdapter;
use crate::ui::Page;
use slint::ComponentHandle as _;

pub fn search_results_init(app: &App) {
}

pub fn search_results(url: &str, app: &App) {
    let app = app.clone();
    let url = url.to_owned();
    std::thread::spawn(move || {
        let url = Url::parse(&url).unwrap();
        let query = url.path_segments().unwrap().next().unwrap();
        let query = percent_encoding::percent_decode_str(query).decode_utf8_lossy().to_string();
        let query = format!("%{}%", query);

        let tracks: Vec<Track> = app.library.query("
            SELECT * 
            FROM Track
            WHERE 
                title LIKE ?1
                OR lyrics LIKE ?1
                OR key LIKE ?1
            ", (query,));
        app.ui.upgrade_in_event_loop(move |ui| {
            // TODO switch to actual search page
            ui.global::<TrackListAdapter>().set_row_data(row_data(&app.library, &tracks));
            ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
            ui.set_page(Page::TrackList);

            // let mut adapter = ui.get_search();
            // let cards: VecModel<CardAdapter> = Default::default();
            // for track in tracks.iter() {
            //     cards.push(track_card(track));
            // }
            // adapter.cards = ModelRc::new(cards);
            // ui.set_search(adapter);
            // ui.set_page(Page::Search);
        }).unwrap();
    });
}

fn row_data(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms as u64))
            .map(|dur| format_length(dur));
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
        .map(|track| track.key.clone().unwrap())
        .map(|key| SharedString::from(key))
        .collect();
    keys.as_slice().into()
}


fn track_card(track: &Track) -> CardAdapter {
    let track = track.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: "Track".into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
    }
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
