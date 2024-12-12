use std::rc::Rc;
use std::time::Duration;

use dimple_core::model::Track;
use slint::ModelRc;
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

        let tracks: Vec<Track> = app.library.query("SELECT * FROM Track
            WHERE artist LIKE ?1 OR album LIKE ?1 OR title LIKE ?1", (query,));
        app.ui.upgrade_in_event_loop(move |ui| {
            // TODO switch to actual search page
            ui.global::<TrackListAdapter>().set_row_data(row_data(&tracks));
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

fn row_data(tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
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
        row.push(StandardListViewItem::from(track.key.unwrap().as_str())); // Key (Hidden)
        row_data.push(row.into());
    }
    row_data.into()
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
