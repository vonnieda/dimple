use dimple_player::player::Player;
use serde::{de, Deserialize, Serialize};
use url::Url;

use std::{collections::VecDeque, rc::Rc, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use dimple_core::{model::{Artist, Medium, Recording, RecordingSource, Release, ReleaseGroup, Track}, source::AccessMode};
use image::DynamicImage;
use slint::{ComponentHandle, Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, StandardListViewItem, TableColumn, VecModel};

slint::include_modules!();
use rayon::prelude::*;

use dimple_librarian::librarian::Librarian;

use directories::ProjectDirs;
use dimple_core::source::Source;

pub struct AppWindowController {
    ui: AppWindow,
    librarian: Arc<Librarian>,
    history: Arc<Mutex<VecDeque<String>>>,
    player: Player,
}

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let dir = dirs.data_dir().to_str().unwrap();
        let librarian = Arc::new(Librarian::new(dir));
        let player = Player::new(librarian.clone());
        Self {
            ui,
            librarian,
            history: Arc::new(Mutex::new(VecDeque::new())),
            player,
        }
    }
}

impl AppWindowController {
    const THUMBNAIL_WIDTH: u32 = 200;
    const THUMBNAIL_HEIGHT: u32 = 200;

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        let history = self.history.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| 
            Self::navigate(url, history.clone(), &librarian.clone(), ui.clone()));

        // let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        self.ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
        // let paths = vec![
        //     "/Users/jason/Music/My Music".to_string(),
        // ];
        // self.librarian.add_library(Box::new(FileLibrary::new(&paths)));
        // self.librarian.add_library(Box::<MusicBrainzLibrary>::default());
        // self.librarian.add_library(Box::<TheAudioDbLibrary>::default());
        // self.librarian.add_library(Box::<FanartTvLibrary>::default());
        // self.librarian.add_library(Box::<DeezerLibrary>::default());
        // self.librarian.add_library(Box::<WikidataLibrary>::default());
        // self.librarian.add_library(Box::<LastFmLibrary>::default());
        // self.librarian.add_library(Box::<CoverArtArchiveLibrary>::default());

        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        self.ui.global::<AppState>().on_set_online(move |online| {
            let librarian = librarian.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let librarian = librarian.clone();
                librarian.set_access_mode(if online { &AccessMode::Online } else { &AccessMode::Offline });
                ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
            }).unwrap();
        });

        // Updates player state
        let ui = self.ui.as_weak();
        let player = self.player.clone();
        thread::spawn(move || {
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = PlayerBarAdapter {
                    duration_seconds: player.duration().as_secs() as i32,
                    duration_label: length_to_string(player.duration().as_secs() as u32).into(),
                    position_seconds: player.position().as_secs() as i32,
                    position_label: length_to_string(player.position().as_secs() as u32).into(),
                    ..Default::default()
                };
                ui.set_player_bar_adapter(adapter);
            }).unwrap();
            thread::sleep(Duration::from_millis(100));
        });
           
        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }

    fn navigate(url: slint::SharedString, history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: slint::Weak<AppWindow>) {

        log::info!("{}", &url);
        // let url = Url::parse(&url);
        if url.starts_with("http") {
            let _ = opener::open_browser(url.to_string());
        }
        else if url.starts_with("dimple://home") {
            Self::home(librarian, ui);
        } 
        else if url.starts_with("dimple://search") {
            Self::search(&url, librarian, ui);
        }
        else if url == "dimple://artists" {
            Self::artists(librarian, ui);
        }
        else if url.starts_with("dimple://artist/") {
            Self::artist_details(&url, librarian, ui);
        }
        else if url.starts_with("dimple://release-group/") {
            Self::release_group_details(&url, librarian, ui);
        }
        else if url.starts_with("dimple://release/") {
            Self::release_details(&url, librarian, ui);
        }
        else if url.starts_with("dimple://recording/") {
            Self::recording_details(&url, librarian, ui);
        }
        else if url == "dimple://back" {
            Self::back(history.clone(), librarian, ui);
        }
        else if url == "dimple://settings" {
            Self::settings(ui);
        }
        else if url == "dimple://refresh" {
            Self::refresh(history.clone(), librarian, ui);
        }

        // Store history.
        if url != "dimple://back" && url != "dimple://refresh" {
            history.lock().unwrap().push_back(url.into());
        }
    }

    fn back(history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: slint::Weak<AppWindow>) {
            let librarian = librarian.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let url: Option<String> = history.lock().ok()
                    .and_then(|mut history| {
                        let _ = history.pop_back()?;
                        history.pop_back()
                    });
                if let Some(url) = url {
                    Self::navigate(url.into(), history.clone(), &librarian, ui.as_weak());
                }
            }).unwrap();
    }

    fn refresh(history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: slint::Weak<AppWindow>) {
            let librarian = librarian.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let url: Option<String> = history.lock().ok()
                    .and_then(|mut history| {
                        history.pop_back()
                    });
                if let Some(url) = url {
                    Self::navigate(url.into(), history.clone(), &librarian, ui.as_weak());
                }
            }).unwrap();
    }

    fn home(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        let librarian = librarian.clone();
        std::thread::spawn(move || {
            let mut tracks: Vec<(Artist, Release, Recording, RecordingSource)> = Vec::new();
            let artists = librarian
                .list(&Artist::default().into(), None, &AccessMode::Online).unwrap()
                .map(Into::<Artist>::into);
            for artist in artists {
                dbg!(artist);
                // for release in artist.releases(librarian.as_ref()) {
                //     for recording in release.recordings(librarian.as_ref()) {
                //         for source in recording.sources(librarian.as_ref()) {
                //             tracks.push((artist.clone(), release.clone(), recording.clone(), source.clone()));
                //         }
                //     }
                // }
            }

            ui.upgrade_in_event_loop(move |ui| {
                let rows: VecModel<ModelRc<StandardListViewItem>> = VecModel::default();
                for (artist, release, recording, source) in tracks {
                    let row = Rc::new(VecModel::default());
                    row.push(recording.title.listview_item());
                    row.push(release.title.listview_item());
                    row.push(artist.name.listview_item());
                    row.push(source.extension.listview_item());
                    row.push(format!("{:?}", source.source_ids).listview_item());
                    row.push("".listview_item());
                    rows.push(row.into());
                }
                let adapter = HomeAdapter {
                    rows: ModelRc::new(rows),
                };
                ui.set_home_adapter(adapter);
                ui.set_page(5);
            }).unwrap();
        });
    }

    fn settings(ui: slint::Weak<AppWindow>) {
        std::thread::spawn(move || {
            // TODO just playing around
            let cache_stats = vec![
                "Metadata Items: 5276 / 27.3MB",
                "Tracks: 986 / 36.2GB",
                "Images: 1286 / 12.6GB",
            ];
            
            ui.upgrade_in_event_loop(move |ui| {
                let cache_stats: Vec<SharedString> = cache_stats.into_iter()
                    .map(Into::into)
                    .collect();
                let adapter = SettingsAdapter {
                    cache_stats: ModelRc::from(cache_stats.as_slice()),
                };
                ui.set_settings_adapter(adapter);
                ui.set_page(6)
            }).unwrap();
        });
    }

    fn search(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        // std::thread::spawn(move || {
        //     ui.upgrade_in_event_loop(move |ui| {
        //         ui.global::<Navigator>().set_busy(true);
        //     }).unwrap();

        //     let url = Url::parse(&url).unwrap();
        //     let query = url.path_segments()
        //         // TODO is this pattern wrong? Shouldn't the or be an error?
        //         .ok_or("missing path").unwrap()
        //         .nth(0)
        //         .ok_or("missing query").unwrap();
        //     let search_results: Vec<_> = librarian.search(query).collect();
        //     // TODO woops, was sorting by name when they are returned by
        //     // relevance. Once more sources are merged I'll need to bring
        //     // rel to the front and sort on it.
        //     // search_results.sort_by_key(|e| e.name().to_lowercase());
        //     let cards = entity_cards(search_results, &librarian, 
        //         Self::THUMBNAIL_WIDTH, 
        //         Self::THUMBNAIL_WIDTH);
        //     ui.upgrade_in_event_loop(move |ui| {
        //         let adapter = CardGridAdapter {
        //             cards: card_adapters(cards),
        //         };
        //         ui.set_card_grid_adapter(adapter);
        //         ui.set_page(0);

        //         ui.global::<Navigator>().set_busy(false);
        //     }).unwrap();
        // });
    }

    fn artists(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        // std::thread::spawn(move || {
        //     let entity = Entities::Artist(Artist::default());
        //     let mut artists: Vec<Artist> = librarian.list(&entity, None)
        //         .filter_map(|e| match e {
        //             Entities::Artist(a) => Some(a),
        //             _ => None,
        //         })
        //         .collect();
        //     artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());
        //     let cards = artist_cards(artists, &librarian,
        //         Self::THUMBNAIL_WIDTH, 
        //         Self::THUMBNAIL_WIDTH);
        //     ui.upgrade_in_event_loop(move |ui| {
        //         let adapter = CardGridAdapter {
        //             cards: card_adapters(cards),
        //         };
        //         ui.set_card_grid_adapter(adapter);
        //         ui.set_page(0);
        //         log::info!("Rendering complete.");
        //     }).unwrap();
        // });
    }

    fn artist_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        // std::thread::spawn(move || {
        //     ui.upgrade_in_event_loop(move |ui| {
        //         ui.global::<Navigator>().set_busy(true);
        //     }).unwrap();
    
        //     let url = Url::parse(&url).unwrap();
        //     let id = url.path_segments()
        //         .ok_or("missing path").unwrap()
        //         .nth(0)
        //         .ok_or("missing id").unwrap();
        //     let artist = Artist::get(id, librarian.as_ref()).unwrap();
        //     let card = entity_card(&Entities::Artist(artist.clone()), 
        //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
        //     let mut release_groups: Vec<_> = artist.release_groups(librarian.as_ref()).collect();
        //     release_groups.sort_by_key(|f| f.first_release_date.to_owned());
        //     release_groups.reverse();
        //     let release_group_cards: Vec<_> = release_groups.par_iter()
        //         .map(|rg| (rg.primary_type.str().to_lowercase().clone(), 
        //             release_group_card(rg, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian)))
        //         .collect();
        //     let album_cards: Vec<_> = release_group_cards.par_iter()
        //         .filter(|(primary_type, _card)| primary_type == "album")
        //         .map(|(_primary_type, card)| card.clone())
        //         .collect();
        //     let single_cards: Vec<_> = release_group_cards.par_iter()
        //         .filter(|(primary_type, _card)| primary_type == "single")
        //         .map(|(_primary_type, card)| card.clone())
        //         .collect();
        //     let ep_cards: Vec<_> = release_group_cards.par_iter()
        //         .filter(|(primary_type, _card)| primary_type == "ep")
        //         .map(|(_primary_type, card)| card.clone())
        //         .collect();
        //     let other_release_group_cards: Vec<_> = release_group_cards.par_iter()
        //         .filter(|(primary_type, _card)| primary_type != "album" && primary_type != "single" && primary_type != "ep")
        //         .map(|(_primary_type, card)| card.clone())
        //         .collect();
        //     let genres: Vec<_> = artist.genres(librarian.as_ref())
        //         .map(|g| Link {
        //             name: g.name.unwrap_or_default(),
        //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
        //         })
        //         .collect();

        //     ui.upgrade_in_event_loop(move |ui| {
        //         let adapter = ArtistDetailsAdapter {
        //             card: card_adapter(&card),
        //             disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
        //             summary: artist.summary.clone().unwrap_or_default().into(),
        //             albums: card_adapters(album_cards),
        //             singles: card_adapters(single_cards),
        //             eps: card_adapters(ep_cards),
        //             others: card_adapters(other_release_group_cards),
        //             genres: link_adapters(genres),
        //             links: link_adapters(artist_links(&artist)),
        //             dump: serde_json::to_string_pretty(&artist).unwrap().into(),
        //             ..Default::default()
        //         };
        //         ui.set_artist_details(adapter);
        //         ui.set_page(1);
        //         ui.global::<Navigator>().set_busy(false);
        //     }).unwrap();
        // });
    }

    fn release_group_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        // std::thread::spawn(move || {
        //     ui.upgrade_in_event_loop(move |ui| {
        //         ui.global::<Navigator>().set_busy(true);
        //     }).unwrap();

        //     let url = Url::parse(&url).unwrap();
        //     let id = url.path_segments()
        //         .ok_or("missing path").unwrap()
        //         .nth(0)
        //         .ok_or("missing id").unwrap();
        //     let release_group = ReleaseGroup::get(id, librarian.as_ref())
        //         .ok_or("release group not found").unwrap();
        //     let card = entity_card(&Entities::ReleaseGroup(release_group.clone()), 
        //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
        //     let mut genres: Vec<_> = release_group.genres(librarian.as_ref())
        //         .map(|g| Link {
        //             name: g.name.unwrap_or_default(),
        //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
        //         })
        //         .collect();
        //     genres.sort_by_key(|g| g.name.to_owned());
        //     let mut artists: Vec<_> = release_group.artists(librarian.as_ref())
        //         .map(|a| Link {
        //             name: a.name.clone().unwrap_or_default(),
        //             url: format!("dimple://artist/{}", a.key.unwrap_or_default()),
        //         })
        //         .collect();
        //     artists.sort_by_key(|a| a.name.to_owned());
        //     let mut releases: Vec<_> = release_group.releases(librarian.as_ref()).collect();
        //     releases.sort_by_key(|r| r.date.clone());
        //     let release_cards = release_cards(releases, &librarian, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT);

        //     ui.upgrade_in_event_loop(move |ui| {
        //         let model = ReleaseGroupDetailsAdapter {                    
        //             card: card_adapter(&card),
        //             disambiguation: release_group.disambiguation.str().into(),
        //             genres: link_adapters(genres),
        //             summary: release_group.summary.str().into(),
        //             primary_type: release_group.primary_type.str().into(),
        //             artists: link_adapters(artists),
        //             links: link_adapters(release_group_links(&release_group)),
        //             // media: media_adapters(release.media),
        //             releases: card_adapters(release_cards),
        //             dump: serde_json::to_string_pretty(&release_group).unwrap().into(),
        //             ..Default::default()
        //         };
        //         ui.set_release_group_details(model);
        //         ui.set_page(2);
        //         ui.global::<Navigator>().set_busy(false);
        //     }).unwrap();
        // });
    }

    fn release_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        // std::thread::spawn(move || {
        //     ui.upgrade_in_event_loop(move |ui| {
        //         ui.global::<Navigator>().set_busy(true);
        //     }).unwrap();

        //     let url = Url::parse(&url).unwrap();
        //     let id = url.path_segments()
        //         .ok_or("missing path").unwrap()
        //         .nth(0)
        //         .ok_or("missing id").unwrap();

        //     let release = Release::get(id, librarian.as_ref())
        //         .ok_or("release not found").unwrap();
        //     let card = entity_card(&Entities::Release(release.clone()), 
        //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);

        //     let mut genres: Vec<_> = release.genres(librarian.as_ref())
        //         .map(|g| Link {
        //             name: g.name.unwrap_or_default(),
        //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
        //         })
        //         .collect();
        //     genres.sort_by_key(|g| g.name.to_owned());

        //     let mut artists: Vec<_> = release.artists(librarian.as_ref())
        //         .map(|a| Link {
        //             name: a.name.clone().unwrap_or_default(),
        //             url: format!("dimple://artist/{}", a.key.unwrap_or_default()),
        //         })
        //         .collect();
        //     artists.sort_by_key(|a| a.name.to_owned());

        //     let recordings: Vec<_> = release.recordings(librarian.as_ref()).collect();
        //     // TODO hmmmmm
        //     let medium = Medium {
        //         tracks: recordings.iter().map(|r| {
        //             let sources = r.sources(librarian.as_ref()).collect();
        //             Track {
        //                 title: r.title.str(),
        //                 length: r.length.unwrap_or_default(),
        //                 recording: r.clone(),
        //                 sources,
        //                 ..Default::default()
        //             }
        //         }).collect(),
        //         ..Default::default()
        //     };
        //     let media = vec![medium];

        //     ui.upgrade_in_event_loop(move |ui| {
        //         let model = ReleaseDetailsAdapter {                    
        //             card: card_adapter(&card),
        //             disambiguation: release.disambiguation.str().into(),
        //             genres: link_adapters(genres),
        //             summary: release.summary.str().into(),
        //             // primary_type: release.primary_type.str().into(),
        //             artists: link_adapters(artists),
        //             links: link_adapters(release_links(&release)),
        //             media: media_adapters(media),
        //             dump: serde_json::to_string_pretty(&release).unwrap().into(),
        //             ..Default::default()
        //         };
        //         ui.set_release_details(model);
        //         ui.set_page(3);
        //         ui.global::<Navigator>().set_busy(false);
        //     }).unwrap();
        // });
    }

    fn recording_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    //     let url = url.to_owned();
    //     std::thread::spawn(move || {
    //         let url = Url::parse(&url).unwrap();
    //         let id = url.path_segments()
    //             .ok_or("missing path").unwrap()
    //             .nth(0)
    //             .ok_or("missing id").unwrap();
    //         let recording = Recording::get(id, librarian.as_ref())
    //             .ok_or("recording not found").unwrap();
    //         let card = entity_card(&Entities::Recording(recording.clone()),
    //             Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
    //         let genres = recording.genres.iter()
    //             .map(|g| Link {
    //                 name: g.name.clone(),
    //                 url: format!("dimple://genre/{}", g.name),
    //             })
    //             .collect();
    //         let artists = recording.artist_credits.iter()
    //             .map(|a| Link {
    //                 name: a.name.clone().unwrap_or_default(),
    //                 url: format!("dimple://artist/{}", a.key),
    //             })
    //             .collect();
    //         let isrcs = recording.isrcs.iter()
    //             .map(|i| Link {
    //                 name: i.to_string(),
    //                 url: format!("https://api.deezer.com/2.0/track/isrc:{}", i),
    //             })
    //             .collect();
    //         // let releases: Vec<_> = release_group.releases.clone();
    //         // let release_cards = release_cards(releases, &librarian, 500, 500);
    //         // let release = release_group.releases.first()
    //         //     .ok_or("no releases")
    //         //     .unwrap();
    //         // let release = release.fetch(librarian.as_ref())
    //         //     .ok_or("release not found")
    //         //     .unwrap();

    //         ui.upgrade_in_event_loop(move |ui| {
    //             let model = RecordingDetailsAdapter {                    
    //                 card: card_adapter(&card),
    //                 disambiguation: recording.disambiguation.clone().into(),
    //                 genres: link_adapters(genres),
    //                 summary: recording.summary.clone().into(),
    //                 // primary_type: recording.primary_type.clone().into(),
    //                 artists: link_adapters(artists),
    //                 links: link_adapters(recording_links(&recording)),
    //                 isrcs: link_adapters(isrcs),
    //                 // media: media_adapters(release.media),
    //                 // releases: card_adapters(release_cards),
    //                 // releases: Default::default()
    //             };
    //             ui.set_recording_details(model);
    //             ui.set_page(4)
    //         }).unwrap();
    //     });
    }
}

// TODO hastily written bunch of adapters, clean em up
fn link_adapters(links: Vec<Link>) -> ModelRc<LinkAdapter> {
    let links: Vec<_> = links.into_iter().map(Into::into).collect();
    ModelRc::from(links.as_slice())
}

// fn entity_cards(entities: Vec<Entities>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| entity_card(ent, width, height, lib))
//         .collect()
// }

// fn entity_card(entity: &Entities, width: u32, height: u32, lib: &Librarian) -> Card {
//     match entity {
//         Entities::Artist(e) => artist_card(e, width, height, lib),
//         Entities::ReleaseGroup(e) => release_group_card(e, width, height, lib),
//         Entities::Release(e) => release_card(e, width, height, lib),
//         Entities::Recording(e) => recording_card(e, width, height, lib),
//         _ => todo!(),
//     }
// }

// fn artist_cards(entities: Vec<Artist>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| artist_card(ent, width, height, lib))
//         .collect()
// }

// fn artist_card(artist: &Artist, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Artist(artist.clone()), width, height),
//             link: Link {
//                 name: artist.name.clone().unwrap_or_default(),
//                 url: format!("dimple://artist/{}", artist.key.str()),
//             },
//         },
//         title: Link {
//             name: artist.name.clone().unwrap_or_default(),
//             url: format!("dimple://artist/{}", artist.key.str()),
//         },
//         sub_title: Link {
//             name: artist.disambiguation.clone().unwrap_or_default(),
//             url: format!("dimple://artist/{}", artist.key.str()),
//         },
//     }
// }

// fn release_group_cards(entities: Vec<ReleaseGroup>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| release_group_card(ent, width, height, lib))
//         .collect()
// }

// fn release_cards(entities: Vec<Release>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| release_card(ent, width, height, lib))
//         .collect()
// }

// fn release_group_card(release_group: &ReleaseGroup, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::ReleaseGroup(release_group.clone()), width, height),
//             link: Link {
//                 name: release_group.title.str(),
//                 url: format!("dimple://release-group/{}", release_group.key.str()),
//             },
//         },
//         title: Link {
//             name: release_group.title.str(),
//             url: format!("dimple://release-group/{}", release_group.key.str()),
//         },
//         sub_title: Link { 
//             name: format!("{:.4} {}", release_group.first_release_date.str(), release_group.primary_type.str()),
//             url: format!("dimple://release-group/{}", release_group.key.str()),
//         },
//     }
// }

// fn release_card(release: &Release, width: u32, height: u32, lib: &Librarian) -> Card {
//     // TODO want to include disambiguation as the title, but also country,
//     // label, and date?
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Release(release.clone()), width, height),
//             link: Link {
//                 name: release.title.str(),
//                 url: format!("dimple://release/{}", release.key.str()),
//             },
//         },
//         title: Link {
//             name: release.disambiguation.clone().str(),
//             url: format!("dimple://release/{}", release.key.str()),
//         },
//         sub_title: Link { 
//             name: format!("{} {}", release.date.str(), release.country.str()),
//             url: format!("dimple://release/{}", release.key.str()),
//         },
//     }
// }

// fn recording_card(recording: &Recording, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Recording(recording.clone()), width, height),
//             link: Link {
//                 name: recording.title.str(),
//                 url: format!("dimple://recording/{}", recording.key.str()),
//             },
//         },
//         title: Link {
//             name: recording.title.str(),
//             url: format!("dimple://release/{}", recording.key.str()),
//         },
//         // TODO
//         // sub_title: 
//         ..Default::default()
//     }
// }

// fn card_adapter(card: &Card) -> CardAdapter {
//     CardAdapter {
//         image: ImageLinkAdapter {
//             // TODO maybe cache, not sure the cost of this.
//             image: dynamic_image_to_slint_image(&card.image.image),
//             name: card.image.link.name.to_owned().into(),
//             url: card.image.link.url.to_owned().into(),
//         },
//         title: card.title.clone().into(),
//         sub_title: card.sub_title.clone().into(),
//     }
// }

// fn artist_links(artist: &Artist) -> Vec<Link> {
//     artist.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
//             url: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
//         }))
//         .collect()
// }

// fn release_group_links(release_group: &ReleaseGroup) -> Vec<Link> {
//     release_group.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
//             url: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
//         }))
//         .collect()
// }

// fn release_links(release: &Release) -> Vec<Link> {
//     release.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
//             url: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
//         }))
//         .collect()
// }

// fn recording_links(recording: &Recording) -> Vec<Link> {
//     recording.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
//             url: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
//         }))
//         .collect()
// }

// fn card_adapters(cards: Vec<Card>) -> ModelRc<CardAdapter> {
//     let card_models: Vec<_>  = cards.iter()
//         .map(card_adapter)
//         .collect();
//     ModelRc::from(card_models.as_slice())
// }

fn length_to_string(length: u32) -> String {
    format!("{}:{:02}", 
        length / (60 * 1000), 
        length % (60 * 1000) / 1000)
}

// fn recording_adapters(recordings: Vec<Recording>) -> ModelRc<TrackAdapter> {
//     let adapters: Vec<_> = recordings.iter()
//         .map(|r| TrackAdapter {
//             title: LinkAdapter {
//                 name: r.title.str(),
//                 url: format!("dimple://recording/{}", r.key.str()).into(),
//             },
//             // track_number: t.number.clone().into(),
//             // length: length_to_string(t.length).into(),
//             artists: Default::default(),
//             plays: 0,
//             ..Default::default()
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

fn track_adapters(tracks: Vec<Track>) -> ModelRc<TrackAdapter> {
    let adapters: Vec<_> = tracks.iter()
        .map(|t| TrackAdapter {
            title: LinkAdapter {
                name: t.title.clone().into(),
                url: format!("dimple://recording/{}", t.recording.key.str()).into(),
            },
            track_number: t.number.clone().into(),
            length: length_to_string(t.length).into(),
            artists: Default::default(),
            plays: 0,
            source_count: t.sources.len() as i32,
        })
        .collect();
    ModelRc::from(adapters.as_slice())
}

fn media_adapters(media: Vec<Medium>) -> ModelRc<MediumAdapter> {
    let adapters: Vec<_> = media.iter()
        .map(|m| MediumAdapter {
            title: format!("{} {} of {}", m.format, m.position, m.disc_count).into(),
            tracks: track_adapters(m.tracks.clone()),
        })
        .collect();
    ModelRc::from(adapters.as_slice())
}

fn dynamic_image_to_slint_image(dynamic_image: &DynamicImage) -> slint::Image {
    let rgba8_image = dynamic_image.clone().into_rgba8();
    let shared_pixbuf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba8_image.as_raw(),
        rgba8_image.width(),
        rgba8_image.height(),
    );
    slint::Image::from_rgba8(shared_pixbuf)
}

// Creates a simple score for a release to use when selecting a
// a default release.
// TODO this is super naive, just needed something to set the example.
fn score_release(r: &Release) -> f64 {
    let mut score = 0.;
    let country = r.country.str().to_lowercase();
    if country == "xw" {
        score += 1.0;
    }                
    else if country == "us" || country == "gb" || country == "xe" {
        score += 0.7;
    }
    else if !country.is_empty() {
        score += 0.1;
    }

    if r.status.str().to_lowercase() == "official" {
        score += 1.0;
    }

    let packaging = r.packaging.str().to_lowercase();
    if packaging == "digipak" {
        score += 1.0;
    }
    else if packaging == "jewelcase" {
        score += 0.5;
    }

    // if !r.media.is_empty() {
    //     let mut media_format_score = 0.;
    //     for media in r.media.clone() {
    //         let format = media.format.to_lowercase();
    //         if format == "digital media" {
    //             media_format_score += 1.0;
    //         }
    //         else if format == "cd" {
    //             media_format_score += 0.5;
    //         }
    //     }
    //     score += media_format_score / r.media.len() as f64;
    // }

    score / 4.
}



#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
struct Link {
    name: String,
    url: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ImageLink {
    link: Link,
    image: DynamicImage,
}

#[derive(Clone, Debug, PartialEq, Default)]
struct Card {
    image: ImageLink,
    title: Link,
    sub_title: Link,
}

impl From<Link> for LinkAdapter {
    fn from(value: Link) -> Self {
        Self {
            name: value.name.into(),
            url: value.url.into(),
        }
    }
}

trait OptStr {
    fn str(&self) -> String;
}

impl OptStr for Option<String> {
    fn str(&self) -> String {
        self.clone().unwrap_or_default()
    }
}

trait AsSharedString {
    fn shared(&self) -> SharedString; 
}

impl AsSharedString for Option<String> {
    fn shared(&self) -> SharedString {
        SharedString::from(self.str())        
    }
}

impl AsSharedString for &str {
    fn shared(&self) -> SharedString {
        SharedString::from(self.to_string())
    }
}

impl AsSharedString for String {
    fn shared(&self) -> SharedString {
        SharedString::from(self)        
    }
}

trait AsListViewItem {
    fn listview_item(&self) -> StandardListViewItem;
}

impl AsListViewItem for Option<String> {
    fn listview_item(&self) -> StandardListViewItem {
        StandardListViewItem::from(self.shared())
    }
}

impl AsListViewItem for &str {
    fn listview_item(&self) -> StandardListViewItem {
        StandardListViewItem::from(self.shared())
    }
}

impl AsListViewItem for String {
    fn listview_item(&self) -> StandardListViewItem {
        StandardListViewItem::from(self.shared())
    }
}

