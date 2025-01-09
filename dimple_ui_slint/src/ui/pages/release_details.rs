use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseDetailsAdapter;
use crate::ui::ImageLinkAdapter;

pub fn release_details_init(app: &App) {
    let app = app.clone();
    let library = app.library.clone();
    // TODO filter events
    library.on_change(Box::new(move |_event| update_model(&app)));
}

pub fn release_details(url: &str, app: &App) {
    let app = app.clone();
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();
    let ui = app.ui.clone();
    ui.upgrade_in_event_loop(move |ui| {
        ui.global::<ReleaseDetailsAdapter>().set_key(key.into());
        update_model(&app);
        ui.set_page(Page::ReleaseDetails);
    }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<ReleaseDetailsAdapter>().get_key();
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let release = Release::get(&library, &key).unwrap();
            let artists = release.artists(&library);
            let genres = release.genres(&library);
            let links = release.links(&library);
            let tracks = release.tracks(&app.library);
            let ui = app.ui.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut card: CardAdapter = release.clone().into();                
                card.image.image = app.images.lazy_get(release.clone(), 275, 275, |ui, image| {
                    let mut card = ui.global::<ReleaseDetailsAdapter>().get_card();
                    card.image.image = image;
                    ui.global::<ReleaseDetailsAdapter>().set_card(card);
                });

                let artists = artist_links(&artists);
                let genres = genre_links(&genres);
                let links: Vec<LinkAdapter> = links.iter().map(|link| {
                        LinkAdapter {
                            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                            url: link.url.clone().into(),
                        }
                    })
                    .collect();
    
                ui.global::<ReleaseDetailsAdapter>().set_card(card.into());
                ui.global::<ReleaseDetailsAdapter>().set_key(release.key.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_release_type(release.release_group_type.clone().unwrap_or("Release".to_string()).into());
                ui.global::<ReleaseDetailsAdapter>().set_artists(ModelRc::from(artists.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_dump(format!("{:?}", release).into());
                ui.global::<ReleaseDetailsAdapter>().set_row_data(row_data(&tracks));
            }).unwrap();
        });
    }).unwrap();
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
        // row.push(track.album.unwrap_or_default().as_str().into()); // Album
        // row.push(track.artist.unwrap_or_default().as_str().into()); // Artist
        row.push("".into()); // Album
        row.push("".into()); // Artist
        row.push(StandardListViewItem::from(length.as_str())); // Length
        row_data.push(row.into());
    }
    row_data.into()
}

fn artist_links(artists: &[Artist]) -> Vec<LinkAdapter> {
    artists.iter().map(|artist| {
        LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}


// fn sort_model(
//     source_model: ModelRc<ModelRc<StandardListViewItem>>,
//     sort_index: i32,
//     sort_ascending: bool,
// ) -> ModelRc<ModelRc<StandardListViewItem>> {
//     let mut model = source_model.clone();

//     if sort_index >= 0 {
//         model = Rc::new(model.clone().sort_by(move |r_a, r_b| {
//             let c_a = r_a.row_data(sort_index as usize).unwrap();
//             let c_b = r_b.row_data(sort_index as usize).unwrap();

//             if sort_ascending {
//                 c_a.text.cmp(&c_b.text)
//             } else {
//                 c_b.text.cmp(&c_a.text)
//             }
//         }))
//         .into();
//     }

//     model
// }

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{}:{:02}", minutes, seconds)
}
