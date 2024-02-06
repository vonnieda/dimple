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

use std::{collections::VecDeque, env, sync::{Arc, Mutex}};

use dimple_core::{library::{Collection, Model}, model::{DimpleArtist, DimpleMedium, DimpleRecording, DimpleRelationContent, DimpleRelease, DimpleReleaseGroup, DimpleTrack}};
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
            "/Users/jason/Music/My Music".to_string(),
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
            let entity = Model::Artist(DimpleArtist::default());
            let mut artists: Vec<DimpleArtist> = librarian.list(&entity)
                .filter_map(|e| match e {
                    Model::Artist(a) => Some(a),
                    _ => None,
                })
                .collect();
            artists.sort_by_key(|a| a.name.to_lowercase());
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
            let artist = DimpleArtist::get(id, librarian.as_ref()).unwrap();
            let card = entity_card(&Model::Artist(artist.clone()), 
                Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
            let mut release_groups = artist.release_groups.clone();
            release_groups.sort_by_key(|f| f.first_release_date.to_owned());
            release_groups.reverse();
            let release_group_cards: Vec<_> = release_groups.par_iter()
                .map(|rg| (rg.primary_type.to_lowercase().clone(), 
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
            let genres = artist.genres.iter()
                .map(|g| Link {
                    name: g.name.clone(),
                    url: format!("dimple://genre/{}", g.name),
                })
                .collect();

            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ArtistDetailsAdapter {
                    card: card_adapter(&card),
                    disambiguation: artist.disambiguation.clone().into(),
                    summary: artist.summary.clone().into(),
                    albums: card_adapters(album_cards),
                    singles: card_adapters(single_cards),
                    eps: card_adapters(ep_cards),
                    others: card_adapters(other_release_group_cards),
                    genres: link_adapters(genres),
                    links: link_adapters(artist_links(&artist)),
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
            let release_group = DimpleReleaseGroup::get(id, librarian.as_ref())
                .ok_or("release group not found").unwrap();
            let card = entity_card(&Model::ReleaseGroup(release_group.clone()), 
                Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
            let mut genres: Vec<_> = release_group.genres.iter()
                .map(|g| Link {
                    name: g.name.clone(),
                    url: format!("dimple://genre/{}", g.name),
                })
                .collect();
            genres.sort_by_key(|g| g.name.to_owned());
            let mut artists: Vec<_> = release_group.artists.iter()
                .map(|a| Link {
                    name: a.name.clone(),
                    url: format!("dimple://artist/{}", a.id),
                })
                .collect();
            artists.sort_by_key(|a| a.name.to_owned());
            let mut releases: Vec<_> = release_group.releases.iter()
                // .filter_map(|f| f.fetch(librarian.as_ref()))
                .collect();
            releases.sort_by(|a, b| {
                let a_score = score_release(a);
                let b_score = score_release(b);
                a_score.partial_cmp(&b_score).unwrap()
            });
            releases.reverse();
            for release in releases.clone() {
                log::info!("{} {} {} {} {} {}", score_release(&release), 
                    release.country, release.status, release.packaging, 
                    release.disambiguation, release.id);
                for media in release.media.clone() {
                    log::info!("  {} {}", media.format, media.disc_count);
                }
            }
            // TODO error handling
            let release = releases.first()
                .ok_or("no releases")
                .unwrap();
            let release = release.fetch(librarian.as_ref())
                .ok_or("release not found")
                .unwrap();

            let mut combined_links = vec![];
            combined_links.extend_from_slice(&release_group_links(&release_group));
            for release in releases {
                let release_links = release_links(&release);
                combined_links.extend_from_slice(&release_links);
            }


            ui.upgrade_in_event_loop(move |ui| {
                let model = ReleaseGroupDetailsAdapter {                    
                    card: card_adapter(&card),
                    disambiguation: release_group.disambiguation.clone().into(),
                    genres: link_adapters(genres),
                    summary: release_group.summary.clone().into(),
                    primary_type: release_group.primary_type.clone().into(),
                    artists: link_adapters(artists),
                    links: link_adapters(combined_links),
                    media: media_adapters(release.media),
                    // releases: card_adapters(release_cards),
                    releases: Default::default()
                };
                ui.set_release_group_details(model);
                ui.set_page(2);
                ui.global::<Navigator>().set_busy(false);
            }).unwrap();
        });
    }

    fn release_details(_url: &str, _librarian: LibrarianHandle, _ui: slint::Weak<AppWindow>) {
        // let url = url.to_string();
        // let ui = ui.clone();
        // std::thread::spawn(move || {
        //     let mbid = url.split_at("dimple://release/".len()).1;
        //     let query = DimpleRelease { id: mbid.to_string(), ..Default::default() };
        //     if let Some(LibraryEntity::Release(rel)) = librarian.fetch(&LibraryEntity::Release(query)) {
        //         ui.upgrade_in_event_loop(move |ui| {
        //             ui.set_release_details((librarian.as_ref(), rel).into());
        //             ui.set_page(3)
        //         }).unwrap();
        //     }
        // });
    }

    fn recording_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            let url = Url::parse(&url).unwrap();
            let id = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing id").unwrap();
            let recording = DimpleRecording::get(id, librarian.as_ref())
                .ok_or("recording not found").unwrap();
            let card = entity_card(&Model::Recording(recording.clone()),
                Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
            let genres = recording.genres.iter()
                .map(|g| Link {
                    name: g.name.clone(),
                    url: format!("dimple://genre/{}", g.name),
                })
                .collect();
            let artists = recording.artist_credits.iter()
                .map(|a| Link {
                    name: a.name.clone(),
                    url: format!("dimple://artist/{}", a.id),
                })
                .collect();
            let isrcs = recording.isrcs.iter()
                .map(|i| Link {
                    name: i.to_string(),
                    url: format!("https://api.deezer.com/2.0/track/isrc:{}", i),
                })
                .collect();
            // let releases: Vec<_> = release_group.releases.clone();
            // let release_cards = release_cards(releases, &librarian, 500, 500);
            // let release = release_group.releases.first()
            //     .ok_or("no releases")
            //     .unwrap();
            // let release = release.fetch(librarian.as_ref())
            //     .ok_or("release not found")
            //     .unwrap();

            ui.upgrade_in_event_loop(move |ui| {
                let model = RecordingDetailsAdapter {                    
                    card: card_adapter(&card),
                    disambiguation: recording.disambiguation.clone().into(),
                    genres: link_adapters(genres),
                    summary: recording.summary.clone().into(),
                    // primary_type: recording.primary_type.clone().into(),
                    artists: link_adapters(artists),
                    links: link_adapters(recording_links(&recording)),
                    isrcs: link_adapters(isrcs),
                    // media: media_adapters(release.media),
                    // releases: card_adapters(release_cards),
                    // releases: Default::default()
                };
                ui.set_recording_details(model);
                ui.set_page(4)
            }).unwrap();
        });
    }
}

// TODO hastily written bunch of adapters, clean em up
fn link_adapters(links: Vec<Link>) -> ModelRc<LinkAdapter> {
    let links: Vec<_> = links.into_iter().map(Into::into).collect();
    ModelRc::from(links.as_slice())
}

fn entity_cards(entities: Vec<Model>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| entity_card(ent, width, height, lib))
        .collect()
}

fn entity_card(entity: &Model, width: u32, height: u32, lib: &Librarian) -> Card {
    match entity {
        Model::Artist(e) => artist_card(e, width, height, lib),
        Model::ReleaseGroup(e) => release_group_card(e, width, height, lib),
        Model::Release(e) => release_card(e, width, height, lib),
        Model::Recording(e) => recording_card(e, width, height, lib),
        _ => todo!(),
    }
}

fn artist_cards(entities: Vec<DimpleArtist>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| artist_card(ent, width, height, lib))
        .collect()
}

fn artist_card(artist: &DimpleArtist, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Model::Artist(artist.clone()), width, height),
            link: Link {
                name: artist.name.clone(),
                url: format!("dimple://artist/{}", artist.id),
            },
        },
        title: Link {
            name: artist.name.clone(),
            url: format!("dimple://artist/{}", artist.id),
        },
        sub_title: Link {
            name: artist.disambiguation.clone(),
            url: format!("dimple://artist/{}", artist.id),
        },
    }
}

fn release_group_cards(entities: Vec<DimpleReleaseGroup>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| release_group_card(ent, width, height, lib))
        .collect()
}

fn release_cards(entities: Vec<DimpleRelease>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| release_card(ent, width, height, lib))
        .collect()
}

fn release_group_card(release_group: &DimpleReleaseGroup, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Model::ReleaseGroup(release_group.clone()), width, height),
            link: Link {
                name: release_group.title.clone(),
                url: format!("dimple://release-group/{}", release_group.id),
            },
        },
        title: Link {
            name: release_group.title.clone(),
            url: format!("dimple://release-group/{}", release_group.id),
        },
        sub_title: Link { 
            name: format!("{:.4} {}", release_group.first_release_date, release_group.primary_type),
            url: format!("dimple://release-group/{}", release_group.id),
        },
    }
}

fn release_card(release: &DimpleRelease, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Model::Release(release.clone()), width, height),
            link: Link {
                name: release.title.clone(),
                url: format!("dimple://release/{}", release.id),
            },
        },
        title: Link {
            name: release.title.clone(),
            url: format!("dimple://release/{}", release.id),
        },
        // TODO
        // sub_title: 
        ..Default::default()
    }
}

fn recording_card(recording: &DimpleRecording, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&Model::Recording(recording.clone()), width, height),
            link: Link {
                name: recording.title.clone(),
                url: format!("dimple://recording/{}", recording.id),
            },
        },
        title: Link {
            name: recording.title.clone(),
            url: format!("dimple://release/{}", recording.id),
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

fn artist_links(artist: &DimpleArtist) -> Vec<Link> {
    artist.relations
        .iter()
        .map(|rel| rel.to_owned())
        // TODO maybe can get name from rel?
        .filter_map(|rel| match rel.content {
            DimpleRelationContent::Url(url) => Some(url),
            _ => None,
        })
        .map(|url| Link {
            name: url.resource.clone(),
            url: url.resource.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/artist/{}", artist.id),
            url: format!("https://musicbrainz.org/artist/{}", artist.id),
        }))
        .collect()
}

fn release_group_links(release_group: &DimpleReleaseGroup) -> Vec<Link> {
    release_group.relations
        .iter()
        .map(|rel| rel.to_owned())
        // TODO maybe can get name from rel?
        .filter_map(|rel| match rel.content {
            DimpleRelationContent::Url(url) => Some(url),
            _ => None,
        })
        .map(|url| Link {
            name: url.resource.clone(),
            url: url.resource.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/release-group/{}", release_group.id),
            url: format!("https://musicbrainz.org/release-group/{}", release_group.id),
        }))
        .collect()
}

fn release_links(release: &DimpleRelease) -> Vec<Link> {
    release.relations
        .iter()
        .map(|rel| rel.to_owned())
        // TODO maybe can get name from rel?
        .filter_map(|rel| match rel.content {
            DimpleRelationContent::Url(url) => Some(url),
            _ => None,
        })
        .map(|url| Link {
            name: url.resource.clone(),
            url: url.resource.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/release/{}", release.id),
            url: format!("https://musicbrainz.org/release/{}", release.id),
        }))
        .collect()
}

fn recording_links(recording: &DimpleRecording) -> Vec<Link> {
    recording.relations
        .iter()
        .map(|rel| rel.to_owned())
        // TODO maybe can get name from rel?
        .filter_map(|rel| match rel.content {
            DimpleRelationContent::Url(url) => Some(url),
            _ => None,
        })
        .map(|url| Link {
            name: url.resource.clone(),
            url: url.resource.clone(),
        })
        .chain(std::iter::once(Link { 
            name: format!("https://musicbrainz.org/recording/{}", recording.id),
            url: format!("https://musicbrainz.org/recording/{}", recording.id),
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

fn track_adapters(tracks: Vec<DimpleTrack>) -> ModelRc<TrackAdapter> {
    let adapters: Vec<_> = tracks.iter()
        .map(|t| TrackAdapter {
            title: LinkAdapter {
                name: t.title.clone().into(),
                url: format!("dimple://recording/{}", t.recording.id).into(),
            },
            track_number: t.number.clone().into(),
            length: length_to_string(t.length).into(),
            artists: Default::default(),
            plays: 0,
        })
        .collect();
    ModelRc::from(adapters.as_slice())
}

fn media_adapters(media: Vec<DimpleMedium>) -> ModelRc<MediumAdapter> {
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
fn score_release(r: &DimpleRelease) -> f64 {
    let mut score = 0.;
    let country = r.country.to_lowercase();
    if country == "xw" {
        score += 1.0;
    }                
    else if country == "us" || country == "gb" || country == "xe" {
        score += 0.7;
    }
    else if !country.is_empty() {
        score += 0.1;
    }

    if r.status.to_lowercase() == "official" {
        score += 1.0;
    }

    let packaging = r.packaging.to_lowercase();
    if packaging == "digipak" {
        score += 1.0;
    }
    else if packaging == "jewelcase" {
        score += 0.5;
    }

    if !r.media.is_empty() {
        let mut media_format_score = 0.;
        for media in r.media.clone() {
            let format = media.format.to_lowercase();
            if format == "digital media" {
                media_format_score += 1.0;
            }
            else if format == "cd" {
                media_format_score += 0.5;
            }
        }
        score += media_format_score / r.media.len() as f64;
    }

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

