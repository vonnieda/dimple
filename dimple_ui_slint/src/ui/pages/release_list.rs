use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Release;
use dimple_core::model::ModelBasics;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

pub fn release_list_init(app: &App) {
    let app = app.clone();
    let library = app.library.clone();
    library.on_change(Box::new(move |_event| update_model(&app)));
}

pub fn release_list(app: &App) {
    update_model(app);
    app.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::ReleaseList)).unwrap();
}

fn update_model(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let library = app.library.clone();
        let releases = Release::list(&library);
        let ui = app.ui.clone();
        let images = app.images.clone();
        ui.upgrade_in_event_loop(move |ui| {
            let cards = release_cards(&images, &releases, &library);
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_release_list(adapter);
        }).unwrap();
    });
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            // card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //     let mut card = ui.get_release_list().cards.row_data(index).unwrap();
            //     card.image.image = image;
            //     ui.get_release_list().cards.set_row_data(index, card);
            // });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

// pub fn track_list_init(app: &App) {
//     let app_ = app.clone();
//     app.ui.upgrade_in_event_loop(move |ui| {
//         let app = app_.clone();
//         ui.global::<TrackListAdapter>().on_sort_table(move |col, ascending| sort_table(&app, col, ascending));
//         let app = app_.clone();
//         ui.global::<TrackListAdapter>().on_play_later(move |key| play_later(&app, &key));
//         let app = app_.clone();
//         ui.global::<TrackListAdapter>().on_play_now(move |key| play_now(&app, &key));
//         let app = app_.clone();
//         ui.global::<TrackListAdapter>().on_play_next(move |key| play_next(&app, &key));
//     }).unwrap();
// }

// pub fn track_list(app: &App) {
//     let app = app.clone();
//     thread::spawn(move || {
//         let tracks = Track::list(&app.library);
//         let library = app.library.clone();
//         app.ui.upgrade_in_event_loop(move |ui| {
//             ui.global::<TrackListAdapter>().set_row_data(row_data(&library, &tracks));
//             ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
//             ui.set_page(Page::TrackList);
//         })
//         .unwrap();
//     });
// }

// fn row_data(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
//     let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
//     for track in tracks {
//         let track = track.clone();
//         let row = Rc::new(VecModel::default());
//         let length = track.length_ms
//             .map(|ms| Duration::from_millis(ms as u64))
//             .map(|dur| format_length(dur));
//         row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
//         row.push(track.album_name(library).unwrap_or_default().as_str().into()); // Album
//         row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
//         row.push(track.media_position.unwrap_or_default().to_string().as_str().into()); // Track #
//         // TODO
//         row.push(0.to_string().as_str().into()); // Plays
//         row.push(length.unwrap_or_default().as_str().into()); // Length
//         row_data.push(row.into());
//     }
//     row_data.into()
// }

// fn row_keys(tracks: &[Track]) -> ModelRc<SharedString> {
//     let keys: Vec<_> = tracks.iter()
//         .map(|track| track.key.clone().unwrap())
//         .map(|key| SharedString::from(key))
//         .collect();
//     keys.as_slice().into()
// }

// fn sort_table(app: &App, col: i32, ascending: bool) {
//     let columns = vec!["title", "album", "artist", "media_position", "plays", "length_ms"];
//     let query = format!("SELECT * FROM Track ORDER BY {} {}", 
//         columns[col as usize], 
//         if ascending { "asc" } else { "desc" });
//     let tracks: Vec<Track> = app.library.query(&query, ());
//     let library = app.library.clone();
//     app.ui.upgrade_in_event_loop(move |ui| {
//         ui.global::<TrackListAdapter>().set_row_data(row_data(&library, &tracks));
//         ui.global::<TrackListAdapter>().set_row_keys(row_keys(&tracks));
//     })
//     .unwrap();
// }

// fn play_now(app: &App, key: &str) {
//     let app = app.clone();
//     app.ui.upgrade_in_event_loop(move |ui| {
//         // TODO think about ephemeral or secondary playlist, or even
//         // a playlist inserted inbetween the playing items
//         // let row_data = ui.global::<TrackListAdapter>().get_row_data();
//         // let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
//         // let key = cell_data.text.as_str();
//         // let play_queue = app.player.queue();
//         // app.library.playlist_insert(&play_queue, &key, row);
//     }).unwrap();
// }

// fn play_next(app: &App, key: &str) {
//     let app = app.clone();
//     app.ui.upgrade_in_event_loop(move |ui| {
//         // TODO think about ephemeral or secondary playlist, or even
//         // a playlist inserted inbetween the playing items
//         // let row_data = ui.global::<TrackListAdapter>().get_row_data();
//         // let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
//         // let key = cell_data.text.as_str();
//         // let play_queue = app.player.queue();
//         // app.library.playlist_insert(&play_queue, &key);
//     }).unwrap();
// }

// fn play_later(app: &App, key: &str) {
//     let app = app.clone();
//     let key = key.to_string();
//     app.ui.upgrade_in_event_loop(move |ui| {
//         // let row_data = ui.global::<TrackListAdapter>().get_row_data();
//         // let cell_data = row_data.row_data(row as usize).unwrap().row_data(6).unwrap();
//         // let key = cell_data.text.as_str();
//         let play_queue = app.player.queue();
//         app.library.playlist_add(&play_queue, &key);
//     }).unwrap();
// }

// fn format_length(length: Duration) -> String {
//     let minutes = length.as_secs() / 60;
//     let seconds = length.as_secs() % 60;
//     format!("{}:{:02}", minutes, seconds)
// }

