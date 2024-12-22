use std::sync::Mutex;

use dimple_core::library::Library;
use dimple_core::merge::CrdtRules;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Track;
use dimple_core::plugins::plugin_host::PluginHost;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
use crate::ui::LinkAdapter;
use slint::ComponentHandle as _;
use crate::ui::CardAdapter;

static CURRENT_TRACK_KEY: Mutex<Option<String>> = Mutex::new(None);

pub fn track_details_init(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app1.clone();
        ui.global::<TrackDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = app1.clone();
        ui.global::<TrackDetailsAdapter>().on_add_to_queue(move |key| add_to_queue(&app, &key));
    }).unwrap();

    let app1 = app.clone();
    app.library.on_change(Box::new(move |event| {
        if let Some(current_track_key) = CURRENT_TRACK_KEY.lock().unwrap().clone() {
            if current_track_key == event.key {
                let track = app1.library.get::<Track>(&event.key).unwrap();
                app1.ui.upgrade_in_event_loop(|ui| {
                    let lyrics = track.lyrics.unwrap_or_default();
                    ui.global::<TrackDetailsAdapter>().set_lyrics(lyrics.into());
                }).unwrap();
            }
        }
    }));
}

pub fn track_details(url: &str, app: &App) {
    let url = url.to_owned();
    let app = app.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let track = app.library.get::<Track>(key).unwrap();
        (*CURRENT_TRACK_KEY.lock().unwrap()) = track.key.clone();

        {
            // TODO temp for testing, not sure where this is gonna live yet.
            let track = track.clone();
            std::thread::spawn(move || {
                refresh_lyrics(&app.library, &app.plugins, &track);
            });
        }

        let artists: Vec<Artist> = vec![Artist {
            // TODO wrong key, just for testing.
            key: track.key.clone(),
            name: track.artist.clone(),
            ..Default::default()
        }];
        let genres: Vec<Genre> = vec![];
        let links: Vec<String> = vec![];

        app.ui.upgrade_in_event_loop(move |ui| {
            let artists: Vec<LinkAdapter> = artists.iter().cloned().map(|artist| {
                LinkAdapter {
                    name: artist.name.unwrap_or_default().into(),
                    url: format!("dimple://artist/{}", artist.key.unwrap()).into(),
                }
            }).collect();

            let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
                LinkAdapter {
                    name: genre.name.unwrap().into(),
                    url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
                }
            }).collect();

            let links: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.into(),
                        url: link.into(),
                    }
                })
                .collect();

            let mut card: CardAdapter = track.clone().into();
            card.image.image = app.images.lazy_get(track.clone(), 275, 275, |ui, image| {
                let mut card = ui.global::<TrackDetailsAdapter>().get_card();
                card.image.image = image;
                ui.global::<TrackDetailsAdapter>().set_card(card);
            });

            ui.global::<TrackDetailsAdapter>().set_card(card.into());
            ui.global::<TrackDetailsAdapter>().set_key(track.key.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_artists(ModelRc::from(artists.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_summary(track.summary.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_lyrics(track.lyrics.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_dump(format!("{:?}", track).into());

            ui.set_page(Page::TrackDetails);
        }).unwrap();
    });
}

fn refresh_lyrics(library: &Library, plugins: &PluginHost, track: &Track) {
    let lyrics = plugins.lyrics(library, track)
        .into_iter()
        .reduce(CrdtRules::merge);
    // TODO txn
    if let Some(mut track) = library.get::<Track>(&track.key.clone().unwrap()) {
        track.lyrics = CrdtRules::merge(track.lyrics, lyrics);
        library.save(&track);
    }
}

fn play_now(app: &App, key: &str) {
    let app = app.clone();
    let key = key.to_string();
    app.ui.upgrade_in_event_loop(move |ui| {
        // TODO think about ephemeral or secondary playlist, or even
        // a playlist inserted inbetween the playing items
        let play_queue = app.player.queue();
        app.library.playlist_add(&play_queue, &key);
        let len = play_queue.len(&app.library);
        app.player.set_queue_index(len - 1);
        app.player.play();
    }).unwrap();
}

fn add_to_queue(app: &App, key: &str) {
    let app = app.clone();
    let key = key.to_string();
    app.ui.upgrade_in_event_loop(move |ui| {
        let play_queue = app.player.queue();
        app.library.playlist_add(&play_queue, &key);
        app.player.play();
    }).unwrap();
}


// struct TrackDetailsController {
//     app: App,
//     key: Option<String>,
// }

// impl TrackDetailsController {
//     pub fn new(app: App) -> Self {
//         let controller = Self {
//             app,
//             key: None,
//         };  
//         {
//             app.ui.upgrade_in_event_loop(move |ui| {
//                 let app1 = app.clone();
//                 ui.global::<TrackDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
//                 let __app = _app.clone();
//                 ui.global::<TrackDetailsAdapter>().on_add_to_queue(move |key| add_to_queue(&__app, &key));
//             }).unwrap();
//         }      
//         controller
//     }

//     pub fn show(&mut self, key: &str) {
//         self.key = Some(key.to_string());
//         self.update_model();
//         self.app.ui.upgrade_in_event_loop(|ui| {
//             ui.set_page(Page::TrackDetails);
//         }).unwrap();
//     }

//     fn update_model(&self) {
//         let library = &self.app.library;
//         let key = self.key.clone().unwrap();
//         let track = library.get::<Track>(&key).unwrap();
//         let artists: Vec<Artist> = todo!();
//         let genres: Vec<Genre> = todo!();
//         let links: Vec<String> = todo!();
//         let images = self.app.images.clone();
//         self.app.ui.upgrade_in_event_loop(move |ui| {
//             let artists: Vec<LinkAdapter> = artists.iter().map(|artist| LinkAdapter::from(artist)).collect();
//             let genres: Vec<LinkAdapter> = genres.iter().map(|genre| LinkAdapter::from(genre)).collect();
//             let links: Vec<LinkAdapter> = links.iter().map(|link| LinkAdapter::from(link)).collect();
    
//             ui.global::<TrackDetailsAdapter>().set_card(CardAdapter::from(&track));
//             ui.global::<TrackDetailsAdapter>().set_key(key.into());
//             ui.global::<TrackDetailsAdapter>().set_artists(artists.as_slice().into());
//             ui.global::<TrackDetailsAdapter>().set_summary(track.summary.clone().unwrap_or_default().into());
//             ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());
//             ui.global::<TrackDetailsAdapter>().set_genres(genres.as_slice().into());
//             ui.global::<TrackDetailsAdapter>().set_lyrics(track.lyrics.clone().unwrap_or_default().into());
//             ui.global::<TrackDetailsAdapter>().set_links(links.as_slice().into());
//             ui.global::<TrackDetailsAdapter>().set_dump(format!("{:?}", track).into());
//         }).unwrap();
//     }

//     fn on_play_now(&self, key: &str) {
//         let player = self.app.player.clone();
//         let key = key.to_string();
//         self.app.ui.upgrade_in_event_loop(move |ui| {
//             player.play_now(&key);
//         }).unwrap();    
//     }
    
//     fn on_play_next(&self, key: &str) {
//         let player = self.app.player.clone();
//         let key = key.to_string();
//         self.app.ui.upgrade_in_event_loop(move |ui| {
//             player.play_next(&key);
//         }).unwrap();    
//     }
    
//     fn on_play_later(&self, key: &str) {
//         let player = self.app.player.clone();
//         let key = key.to_string();
//         self.app.ui.upgrade_in_event_loop(move |ui| {
//             player.play_later(&key);
//         }).unwrap();    
//     }    

//     fn on_track_changed(&self) {
//         // let _app = app.clone();
//         // app.library.on_change(Box::new(move |key| {
//         //     let __app = _app.clone();
//         //     _app.ui.upgrade_in_event_loop(move |ui| {
//         //         // TODO should check key
//         //         // TODO don't want to add to the event loop on every library change
//         //         // and then check the key, so it's probably time to keep some state
//         //         // on this side, too, like the key.
//         //         if ui.get_page() == Page::TrackDetails {
//         //             __app.refresh();
//         //         }
//         //     }).unwrap();
//         // }));
//     }
// }

// impl From<&Artist> for LinkAdapter {
//     fn from(value: &Artist) -> Self {
//         todo!()
//     }
// }

// impl From<&Genre> for LinkAdapter {
//     fn from(value: &Genre) -> Self {
//         todo!()
//     }
// }

// impl From<&String> for LinkAdapter {
//     fn from(value: &String) -> Self {
//         todo!()
//     }
// }

// impl From<&Track> for CardAdapter {
//     fn from(value: &Track) -> Self {
//         todo!()
//     }
// }


