use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_deezer_library::DeezerLibrary;
use dimple_wikidata_library::WikidataLibrary;
use serde::{Deserialize, Serialize};
use url::Url;

use std::{sync::{Arc, RwLock}, error::Error, collections::HashMap};

use dimple_core::{model::{DimpleArtist, DimpleReleaseGroup, DimpleRelationContent, DimpleRelease, DimpleMedium, DimpleTrack}, library::{Library, LibraryEntity}};
use dimple_librarian::librarian::{Librarian, self};
use image::DynamicImage;
use slint::{ModelRc, SharedPixelBuffer, Rgba8Pixel, ComponentHandle, Model, VecModel};

slint::include_modules!();
use rayon::prelude::*;

pub type LibrarianHandle = Arc<Librarian>;

pub struct AppWindowController {
    ui: AppWindow,
    librarian: LibrarianHandle,
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
            // player,
        }
    }
}

impl AppWindowController {
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        
        self.ui.global::<Navigator>().on_navigate(move |url| 
            Self::navigate(url, librarian.clone(), ui.clone()));

        // self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music")));
        self.librarian.add_library(Box::<MusicBrainzLibrary>::default());
        self.librarian.add_library(Box::<LastFmLibrary>::default());
        self.librarian.add_library(Box::<FanartTvLibrary>::default());
        self.librarian.add_library(Box::<DeezerLibrary>::default());
        self.librarian.add_library(Box::<WikidataLibrary>::default());

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }

    fn navigate(url: slint::SharedString, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        dbg!(&url);
        // let url = Url::parse(&url);
        if url.starts_with("http") {
            opener::open_browser(url.to_string());
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
    }

    fn home(ui: slint::Weak<AppWindow>) {
        ui.upgrade_in_event_loop(move |ui| {
            let adapter = CardGridAdapter::default();
            ui.set_card_grid_adapter(adapter);
            ui.set_page(0)
        }).unwrap();
    }

    fn search(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            let url = Url::parse(&url).unwrap();
            let query = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing query").unwrap();
            let mut search_results: Vec<_> = librarian.search(query).collect();
            search_results.sort_by_key(|e| e.name().to_lowercase());
            let cards = entity_cards(search_results, &librarian, 500, 500);
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = CardGridAdapter {
                    cards: card_adapters(cards),
                };
                ui.set_card_grid_adapter(adapter);
                ui.set_page(0)
            }).unwrap();
        });
    }

    fn artists(librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        std::thread::spawn(move || {
            let mut artists: Vec<DimpleArtist> = librarian.artists().collect();
            artists.sort_by_key(|a| a.name.to_lowercase());
            let cards = artist_cards(artists, &librarian, 500, 500);
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
            let url = Url::parse(&url).unwrap();
            let id = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing id").unwrap();
            let artist = DimpleArtist::get(id, librarian.as_ref()).unwrap();
            let card = entity_card(&LibraryEntity::Artist(artist.clone()), 500, 500, &librarian);
            let mut release_groups = artist.release_groups.clone();
            release_groups.sort_by_key(|f| f.first_release_date.to_owned());
            release_groups.reverse();
            let albums: Vec<_> = release_groups.iter()
                .filter(|f| f.primary_type.to_lowercase() == "album")
                .cloned()
                .collect();
            let album_cards = release_group_cards(albums, &librarian, 500, 500);
            let singles: Vec<_> = release_groups.iter()
                .filter(|f| f.primary_type.to_lowercase() == "single")
                .cloned()
                .collect();
            let single_cards = release_group_cards(singles, &librarian, 500, 500);
            let eps: Vec<_> = release_groups.iter()
                .filter(|f| f.primary_type.to_lowercase() == "ep")
                .cloned()
                .collect();
            let ep_cards = release_group_cards(eps, &librarian, 500, 500);
            let other_release_groups: Vec<_> = release_groups.iter()
                .filter(|f| f.primary_type.to_lowercase() != "album" && f.primary_type.to_lowercase() != "single" && f.primary_type.to_lowercase() != "ep")
                .cloned()
                .collect();
            let other_release_group_cards = release_group_cards(other_release_groups, &librarian, 500, 500);
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
                ui.set_page(1)
            }).unwrap();
        });
    }

    fn release_group_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        std::thread::spawn(move || {
            let url = Url::parse(&url).unwrap();
            let id = url.path_segments()
                .ok_or("missing path").unwrap()
                .nth(0)
                .ok_or("missing id").unwrap();
            let release_group = DimpleReleaseGroup::get(id, librarian.as_ref())
                .ok_or("release group not found").unwrap();
            let card = entity_card(&LibraryEntity::ReleaseGroup(release_group.clone()), 500, 500, &librarian);
            let genres = release_group.genres.iter()
                .map(|g| Link {
                    name: g.name.clone(),
                    url: format!("dimple://genre/{}", g.name),
                })
                .collect();
            let artists = release_group.artists.iter()
                .map(|a| Link {
                    name: a.name.clone(),
                    url: format!("dimple://artist/{}", a.id),
                })
                .collect();
            let releases: Vec<_> = release_group.releases.clone();
            let release_cards = release_cards(releases, &librarian, 500, 500);
            let release = release_group.releases.first()
                .ok_or("no releases")
                .unwrap();
            let release = release.fetch(librarian.as_ref())
                .ok_or("release not found")
                .unwrap();

            ui.upgrade_in_event_loop(move |ui| {
                let model = ReleaseGroupDetailsAdapter {                    
                    card: card_adapter(&card),
                    disambiguation: release_group.disambiguation.clone().into(),
                    genres: link_adapters(genres),
                    summary: release_group.summary.clone().into(),
                    primary_type: release_group.primary_type.clone().into(),
                    artists: link_adapters(artists),
                    links: link_adapters(release_group_links(&release_group)),
                    media: media_adapters(release.media),
                    releases: card_adapters(release_cards),
                };
                ui.set_release_group_details(model);
                ui.set_page(2)
            }).unwrap();
        });
    }

    fn release_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
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
}

fn link_adapter(link: &Link) -> LinkAdapter {
    LinkAdapter {
        name: link.name.clone().into(),
        url: link.url.clone().into(),
    }
}

fn link_adapters(links: Vec<Link>) -> ModelRc<LinkAdapter> {
    let links: Vec<_> = links.iter()
        .map(link_adapter)
        .collect();
    ModelRc::from(links.as_slice())
}

fn entity_cards(entities: Vec<LibraryEntity>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
    entities.par_iter()
        .map(|ent| entity_card(ent, width, height, lib))
        .collect()
}

fn entity_card(entity: &LibraryEntity, width: u32, height: u32, lib: &Librarian) -> Card {
    match entity {
        LibraryEntity::Artist(e) => artist_card(e, width, height, lib),
        LibraryEntity::ReleaseGroup(e) => release_group_card(e, width, height, lib),
        LibraryEntity::Release(e) => release_card(e, width, height, lib),
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
            image: lib.thumbnail(&LibraryEntity::Artist(artist.clone()), width, height),
            link: Link {
                name: artist.name.clone(),
                url: format!("dimple://artist/{}", artist.id),
            },
        },
        title: Link {
            name: artist.name.clone(),
            url: format!("dimple://artist/{}", artist.id),
        },
        // TODO
        // sub_title: 
        ..Default::default()
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
            image: lib.thumbnail(&LibraryEntity::ReleaseGroup(release_group.clone()), width, height),
            link: Link {
                name: release_group.title.clone(),
                url: format!("dimple://release-group/{}", release_group.id),
            },
        },
        title: Link {
            name: release_group.title.clone(),
            url: format!("dimple://release-group/{}", release_group.id),
        },
        // TODO
        // sub_title: 
        ..Default::default()
    }
}

fn release_card(release: &DimpleRelease, width: u32, height: u32, lib: &Librarian) -> Card {
    Card {
        image: ImageLink {
            image: lib.thumbnail(&LibraryEntity::Release(release.clone()), width, height),
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

fn card_adapter(card: &Card) -> CardAdapter {
    CardAdapter {
        image: ImageLinkAdapter {
            // TODO maybe cache, not sure the cost of this.
            image: dynamic_image_to_slint_image(&card.image.image),
            name: card.image.link.name.to_owned().into(),
            url: card.image.link.url.to_owned().into(),
        },
        title: LinkAdapter {
            name: card.title.name.to_owned().into(),
            url: card.title.url.to_owned().into(),
        },
        // TODO
        // sub_title: LinkAdapter {
        //     name: card.title.name.to_owned().into(),
        //     url: card.title.url.to_owned().into(),
        // },
        ..Default::default()
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

fn card_adapters(cards: Vec<Card>) -> ModelRc<CardAdapter> {
    let card_models: Vec<_>  = cards.iter()
        .map(card_adapter)
        .collect();
    ModelRc::from(card_models.as_slice())
}

fn track_adapters(tracks: Vec<DimpleTrack>) -> ModelRc<TrackAdapter> {
    let adapters: Vec<_> = tracks.iter()
        .map(|t| TrackAdapter {
            title: LinkAdapter {
                name: t.title.clone().into(),
                url: format!("dimple://track/{}", t.id).into(),
            },
            track_number: t.number.clone().into(),
            length: format!("{}:{:02}", t.length, t.length).into(),
            plays: 17,
            ..Default::default()
        })
        .collect();
    ModelRc::from(adapters.as_slice())
}

fn media_adapters(media: Vec<DimpleMedium>) -> ModelRc<MediumAdapter> {
    let adapters: Vec<_> = media.iter()
        .map(|m| MediumAdapter {
            title: format!("{} {} of {}", m.format, m.position, m.disc_count).into(),
            tracks: track_adapters(m.tracks.clone()),
            ..Default::default()
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
    sub_title: Vec<Link>,
}