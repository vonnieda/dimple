use dimple_coverartarchive_library::CoverArtArchiveLibrary;
use dimple_file_library::dimple_file_library::FileLibrary;
use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_deezer_library::DeezerLibrary;
use dimple_theaudiodb_library::TheAudioDbLibrary;
use dimple_wikidata_library::WikidataLibrary;
use serde::{Deserialize, Serialize};
use url::Url;

use std::{collections::VecDeque, env, path::Display, sync::{Arc, Mutex}};

use dimple_core::{collection::Collection, model::{Artist, Entities, Entity, Recording, Release, ReleaseGroup}};
use dimple_librarian::librarian::{Librarian};
use image::DynamicImage;
use slint::{ModelRc, SharedPixelBuffer, Rgba8Pixel, ComponentHandle, SharedString};

slint::include_modules!();
use rayon::prelude::*;

pub type LibrarianHandle = Arc<Librarian>;

pub struct AppWindowController {
    ui: AppWindow,
    librarian: LibrarianHandle,
    history: Arc<Mutex<VecDeque<String>>>,
    // player: PlayerHandle,
}

use directories::ProjectDirs;

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let dir = dirs.data_dir().to_str().unwrap();
        let librarian = Arc::new(Librarian::new(dir));
        // let player = Player::new(librarian.clone());
        Self {
            ui,
            librarian,
            history: Arc::new(Mutex::new(VecDeque::new())),
            // player,
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
            Self::navigate(url, history.clone(), librarian.clone(), ui.clone()));

        let paths = vec![
            // "/Users/jason/Music/My Music".to_string(),
            "/Users/jason/Music/Dimple Test Tracks".to_string(),
        ];
        self.librarian.add_library(Box::new(FileLibrary::new(&paths)));
        self.librarian.add_library(Box::<MusicBrainzLibrary>::default());
        self.librarian.add_library(Box::new(TheAudioDbLibrary::default()));
        self.librarian.add_library(Box::new(FanartTvLibrary::default()));
        self.librarian.add_library(Box::<DeezerLibrary>::default());
        self.librarian.add_library(Box::<WikidataLibrary>::default());
        self.librarian.add_library(Box::<LastFmLibrary>::default());
        self.librarian.add_library(Box::<CoverArtArchiveLibrary>::default());

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }

    fn navigate(url: slint::SharedString, history: Arc<Mutex<VecDeque<String>>>, 
        librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {

        log::info!("{}", &url);
        // let url = Url::parse(&url);
        if url.starts_with("http") {
            let _ = opener::open_browser(url.to_string());
        }
        else if url.starts_with("dimple://home") {
            Self::home(ui);
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
        librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
            ui.upgrade_in_event_loop(move |ui| {
                let url: Option<String> = history.lock().ok()
                    .and_then(|mut history| {
                        let _ = history.pop_back()?;
                        history.pop_back()
                    });
                if let Some(url) = url {
                    Self::navigate(url.into(), history.clone(), librarian, ui.as_weak());
                }
            }).unwrap();
    }

    fn refresh(history: Arc<Mutex<VecDeque<String>>>, 
        librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
            ui.upgrade_in_event_loop(move |ui| {
                let url: Option<String> = history.lock().ok()
                    .and_then(|mut history| {
                        history.pop_back()
                    });
                if let Some(url) = url {
                    Self::navigate(url.into(), history.clone(), librarian, ui.as_weak());
                }
            }).unwrap();
    }

    fn home(ui: slint::Weak<AppWindow>) {
        ui.upgrade_in_event_loop(move |ui| {
            // let adapter = CardGridAdapter::default();
            // ui.set_card_grid_adapter(adapter);
            ui.set_page(5)
        }).unwrap();
    }

    fn settings(ui: slint::Weak<AppWindow>) {
        std::thread::spawn(move || {
            // TODO just playing around
            let cache_stats = vec![
                "Metadata Objects: 5276 / 27.3MB",
                "Images: 1286 / 12.6GB",
                "Audio Files: 986 / 36.2GB",
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

    fn search(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Navigator>().set_busy(true);
            }).unwrap();

            let url = Url::parse(&url).unwrap();
            let query = url.path_segments()
                // TODO is this pattern wrong? Shouldn't the or be an error?
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing query").unwrap();
            let search_results: Vec<_> = librarian.search(query).collect();
            // TODO woops, was sorting by name when they are returned by
            // relevance. Once more sources are merged I'll need to bring
            // rel to the front and sort on it.
            // search_results.sort_by_key(|e| e.name().to_lowercase());
            let cards = entity_cards(search_results, &librarian, 
                Self::THUMBNAIL_WIDTH, 
                Self::THUMBNAIL_WIDTH);
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = CardGridAdapter {
                    cards: card_adapters(cards),
                };
                ui.set_card_grid_adapter(adapter);
                ui.set_page(0);

                ui.global::<Navigator>().set_busy(false);
            }).unwrap();
        });
    }

    fn artists(librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        std::thread::spawn(move || {
            let entity = Entities::Artist(Artist::default());
            let mut artists: Vec<Artist> = librarian.list(&entity, None)
                .filter_map(|e| match e {
                    Entities::Artist(a) => Some(a),
                    _ => None,
                })
                .collect();
            artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());
            let cards = artist_cards(artists, &librarian,
                Self::THUMBNAIL_WIDTH, 
                Self::THUMBNAIL_WIDTH);
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = CardGridAdapter {
                    cards: card_adapters(cards),
                };
                ui.set_card_grid_adapter(adapter);
                ui.set_page(0)
            }).unwrap();
        });
    }

    fn artist_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Navigator>().set_busy(true);
            }).unwrap();
    
            let url = Url::parse(&url).unwrap();
            let id = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing id").unwrap();
            let artist = Artist::get(id, librarian.as_ref()).unwrap();
            let card = entity_card(&Entities::Artist(artist.clone()), 
                Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
            let mut release_groups: Vec<_> = artist.release_groups(librarian.as_ref()).collect();
            release_groups.sort_by_key(|f| f.first_release_date.to_owned());
            release_groups.reverse();
            let release_group_cards: Vec<_> = release_groups.par_iter()
                .map(|rg| (rg.primary_type.str().to_lowercase().clone(), 
                    release_group_card(rg, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian)))
                .collect();
            let album_cards: Vec<_> = release_group_cards.par_iter()
                .filter(|(primary_type, _card)| primary_type == "album")
                .map(|(_primary_type, card)| card.clone())
                .collect();
            let single_cards: Vec<_> = release_group_cards.par_iter()
                .filter(|(primary_type, _card)| primary_type == "single")
                .map(|(_primary_type, card)| card.clone())
                .collect();
            let ep_cards: Vec<_> = release_group_cards.par_iter()
                .filter(|(primary_type, _card)| primary_type == "ep")
                .map(|(_primary_type, card)| card.clone())
                .collect();
            let other_release_group_cards: Vec<_> = release_group_cards.par_iter()
                .filter(|(primary_type, _card)| primary_type != "album" && primary_type != "single" && primary_type != "ep")
                .map(|(_primary_type, card)| card.clone())
                .collect();
            // let genres = artist.genres.iter()
            //     .map(|g| Link {
            //         name: g.name.clone(),
            //         url: format!("dimple://genre/{}", g.name),
            //     })
            //     .collect();

            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ArtistDetailsAdapter {
                    card: card_adapter(&card),
                    disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
                    summary: artist.summary.clone().unwrap_or_default().into(),
                    albums: card_adapters(album_cards),
                    singles: card_adapters(single_cards),
                    eps: card_adapters(ep_cards),
                    others: card_adapters(other_release_group_cards),
                    // genres: link_adapters(genres),
                    links: link_adapters(artist_links(&artist)),
                    ..Default::default()
                };
                ui.set_artist_details(adapter);
                ui.set_page(1);
                ui.global::<Navigator>().set_busy(false);
            }).unwrap();
        });
    }

    fn release_group_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Navigator>().set_busy(true);
            }).unwrap();

            let url = Url::parse(&url).unwrap();
            let id = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing id").unwrap();
            let release_group = ReleaseGroup::get(id, librarian.as_ref())
                .ok_or("release group not found").unwrap();
            let card = entity_card(&Entities::ReleaseGroup(release_group.clone()), 
                Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
            // let mut genres: Vec<_> = release_group.genres.iter()
            //     .map(|g| Link {
            //         name: g.name.clone(),
            //         url: format!("dimple://genre/{}", g.name),
            //     })
            //     .collect();
            // genres.sort_by_key(|g| g.name.to_owned());
            // let mut artists: Vec<_> = release_group.artists.iter()
            //     .map(|a| Link {
            //         name: a.name.clone().unwrap_or_default(),
            //         url: format!("dimple://artist/{}", a.key),
            //     })
            //     .collect();
            // artists.sort_by_key(|a| a.name.to_owned());
            let releases: Vec<_> = release_group.releases(librarian.as_ref()).collect();
            let release_cards = release_cards(releases, &librarian, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT);

            ui.upgrade_in_event_loop(move |ui| {
                let model = ReleaseGroupDetailsAdapter {                    
                    card: card_adapter(&card),
                    disambiguation: release_group.disambiguation.str().into(),
                    // genres: link_adapters(genres),
                    summary: release_group.summary.str().into(),
                    primary_type: release_group.primary_type.str().into(),
                    // artists: link_adapters(artists),
                    links: link_adapters(release_group_links(&release_group)),
                    // media: media_adapters(release.media),
                    releases: card_adapters(release_cards),
                    ..Default::default()
                };
                ui.set_release_group_details(model);
                ui.set_page(2);
                ui.global::<Navigator>().set_busy(false);
            }).unwrap();
        });
    }

    fn release_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Navigator>().set_busy(true);
            }).unwrap();

            let url = Url::parse(&url).unwrap();
            let id = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing id").unwrap();
            let release = Release::get(id, librarian.as_ref())
                .ok_or("release not found").unwrap();
            let card = entity_card(&Entities::Release(release.clone()), 
                Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
            let recordings = release.recordings(librarian.as_ref());
            // let mut genres: Vec<_> = release_group.genres.iter()
            //     .map(|g| Link {
            //         name: g.name.clone(),
            //         url: format!("dimple://genre/{}", g.name),
            //     })
            //     .collect();
            // genres.sort_by_key(|g| g.name.to_owned());
            // let mut artists: Vec<_> = release_group.artists.iter()
            //     .map(|a| Link {
            //         name: a.name.clone().unwrap_or_default(),
            //         url: format!("dimple://artist/{}", a.key),
            //     })
            //     .collect();
            // artists.sort_by_key(|a| a.name.to_owned());
            // let releases: Vec<_> = release_group.releases(librarian.as_ref()).collect();
            // let release_cards = release_cards(releases, &librarian, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT);

            ui.upgrade_in_event_loop(move |ui| {
                let model = ReleaseDetailsAdapter {                    
                    card: card_adapter(&card),
                    disambiguation: release.disambiguation.str().into(),
                    // genres: link_adapters(genres),
                    summary: release.summary.str().into(),
                    // primary_type: release.primary_type.str().into(),
                    // artists: link_adapters(artists),
                    links: link_adapters(release_links(&release)),
                    // media: media_adapters(release.media),
                    // releases: card_adapters(release_cards),
                    ..Default::default()
                };
                ui.set_release_details(model);
                ui.set_page(3);
                ui.global::<Navigator>().set_busy(false);
            }).unwrap();
        });
    }

    fn recording_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
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

fn entity_cards(entities: Vec<Entities>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| entity_card(ent, width, height, lib))
        .collect()
}

fn entity_card(entity: &Entities, width: u32, height: u32, lib: &Librarian) -> Card {
    match entity {
        Entities::Artist(e) => artist_card(e, width, height, lib),
        Entities::ReleaseGroup(e) => release_group_card(e, width, height, lib),
        Entities::Release(e) => release_card(e, width, height, lib),
        Entities::Recording(e) => recording_card(e, width, height, lib),
        _ => todo!(),
    }
}

fn artist_cards(entities: Vec<Artist>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| artist_card(ent, width, height, lib))
        .collect()
}

fn artist_card(artist: &Artist, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Entities::Artist(artist.clone()), width, height),
            link: Link {
                name: artist.name.clone().unwrap_or_default(),
                url: format!("dimple://artist/{}", artist.key.str()),
            },
        },
        title: Link {
            name: artist.name.clone().unwrap_or_default(),
            url: format!("dimple://artist/{}", artist.key.str()),
        },
        sub_title: Link {
            name: artist.disambiguation.clone().unwrap_or_default(),
            url: format!("dimple://artist/{}", artist.key.str()),
        },
    }
}

fn release_group_cards(entities: Vec<ReleaseGroup>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| release_group_card(ent, width, height, lib))
        .collect()
}

fn release_cards(entities: Vec<Release>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| release_card(ent, width, height, lib))
        .collect()
}

fn release_group_card(release_group: &ReleaseGroup, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Entities::ReleaseGroup(release_group.clone()), width, height),
            link: Link {
                name: release_group.title.str(),
                url: format!("dimple://release-group/{}", release_group.key.str()),
            },
        },
        title: Link {
            name: release_group.title.str(),
            url: format!("dimple://release-group/{}", release_group.key.str()),
        },
        sub_title: Link { 
            name: format!("{:.4} {}", release_group.first_release_date.str(), release_group.primary_type.str()),
            url: format!("dimple://release-group/{}", release_group.key.str()),
        },
    }
}

fn release_card(release: &Release, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Entities::Release(release.clone()), width, height),
            link: Link {
                name: release.title.str(),
                url: format!("dimple://release/{}", release.key.str()),
            },
        },
        title: Link {
            name: release.title.clone().str(),
            url: format!("dimple://release/{}", release.key.str()),
        },
        sub_title: Link { 
            name: format!("{:.4} {}", release.date.str(), release.country.str()),
            url: format!("dimple://release/{}", release.key.str()),
        },
        ..Default::default()
    }
}

fn recording_card(recording: &Recording, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Entities::Recording(recording.clone()), width, height),
            link: Link {
                name: recording.title.str(),
                url: format!("dimple://recording/{}", recording.key.str()),
            },
        },
        title: Link {
            name: recording.title.str(),
            url: format!("dimple://release/{}", recording.key.str()),
        },
        // TODO
        // sub_title: 
        ..Default::default()
    }
}

fn card_adapter(card: &Card) -> CardAdapter {
    CardAdapter {
        image: ImageLinkAdapter {
            // TODO maybe cache, not sure the cost of this.
            image: dynamic_image_to_slint_image(&card.image.image),
            name: card.image.link.name.to_owned().into(),
            url: card.image.link.url.to_owned().into(),
        },
        title: card.title.clone().into(),
        sub_title: card.sub_title.clone().into(),
    }
}

fn artist_links(artist: &Artist) -> Vec<Link> {
    artist.links
        .iter()
        .map(|url| Link {
            name: url.clone(),
            url: url.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
            url: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
        }))
        .collect()
}

fn release_group_links(release_group: &ReleaseGroup) -> Vec<Link> {
    release_group.links
        .iter()
        .map(|url| Link {
            name: url.clone(),
            url: url.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
            url: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
        }))
        .collect()
}

fn release_links(release: &Release) -> Vec<Link> {
    release.links
        .iter()
        .map(|url| Link {
            name: url.clone(),
            url: url.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
            url: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
        }))
        .collect()
}

fn recording_links(recording: &Recording) -> Vec<Link> {
    recording.links
        .iter()
        .map(|url| Link {
            name: url.clone(),
            url: url.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
            url: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
        }))
        .collect()
}

fn card_adapters(cards: Vec<Card>) -> ModelRc<CardAdapter> {
    let card_models: Vec<_>  = cards.iter()
        .map(card_adapter)
        .collect();
    ModelRc::from(card_models.as_slice())
}

fn length_to_string(length: u32) -> String {
    format!("{}:{:02}", 
        length / (60 * 1000), 
        length % (60 * 1000) / 1000)
}

// fn track_adapters(tracks: Vec<Track>) -> ModelRc<TrackAdapter> {
//     let adapters: Vec<_> = tracks.iter()
//         .map(|t| TrackAdapter {
//             title: LinkAdapter {
//                 name: t.title.clone().into(),
//                 url: format!("dimple://recording/{}", t.recording.key).into(),
//             },
//             track_number: t.number.clone().into(),
//             length: length_to_string(t.length).into(),
//             artists: Default::default(),
//             plays: 0,
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// fn media_adapters(media: Vec<Medium>) -> ModelRc<MediumAdapter> {
//     let adapters: Vec<_> = media.iter()
//         .map(|m| MediumAdapter {
//             title: format!("{} {} of {}", m.format, m.position, m.disc_count).into(),
//             tracks: track_adapters(m.tracks.clone()),
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

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