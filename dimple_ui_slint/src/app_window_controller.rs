use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_deezer_library::DeezerLibrary;
use dimple_wikidata_library::WikidataLibrary;

use std::sync::Arc;

use dimple_core::{model::{DimpleArtist, DimpleGenre, DimpleTrack, DimpleReleaseGroup, DimpleRelationContent, DimpleRelease}, library::{Library, LibraryEntity}};
use dimple_librarian::librarian::{Librarian, self};
use image::DynamicImage;
use slint::{ModelRc, SharedPixelBuffer, Rgba8Pixel, ComponentHandle, Model};

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
            ui.set_card_grid_cards(ModelRc::from(vec![].as_slice()));
            ui.set_page(0)
        }).unwrap();
    }

    fn search(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_string();
        std::thread::spawn(move || {
            let query_str = url.split_at("dimple://search".len()).1;
            let mut search_results: Vec<LibraryEntity> = librarian.search(query_str).collect();

            // Preload images
            // TODO is trash
            search_results
                .par_iter()
                .for_each(|f| {
                    thumbnail(&librarian, f, 200, 200);
                    thumbnail(&librarian, f, 275, 275);
                });

            search_results.sort_by_key(|e| e.name().to_lowercase());
            ui.upgrade_in_event_loop(move |ui| {
                let cards: Vec<CardModel> = search_results.into_iter()
                    .map(|a| (librarian.as_ref(), a))
                    .map(Into::into)
                    .collect();
                ui.set_card_grid_cards(ModelRc::from(cards.as_slice()));
                ui.set_page(0)
            }).unwrap();
        });
    }

    fn artists(librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        std::thread::spawn(move || {
            let mut artists: Vec<DimpleArtist> = librarian.artists().collect();
            artists.sort_by_key(|a| a.name.to_lowercase());
            ui.upgrade_in_event_loop(move |ui| {
                let cards: Vec<CardModel> = artists.into_iter()
                    .map(|a| (librarian.as_ref(), a))
                    .map(Into::into)
                    .collect();
                ui.set_card_grid_cards(ModelRc::from(cards.as_slice()));
                ui.set_page(0)
            }).unwrap();
        });
    }

    fn artist_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_string();
        let ui = ui.clone();
        std::thread::spawn(move || {
            // TODO ew
            let mbid = url.split_at("dimple://artist/".len()).1;
            let query = DimpleArtist { id: mbid.to_string(), ..Default::default() };
            if let Some(LibraryEntity::Artist(artist)) = librarian.fetch(&LibraryEntity::Artist(query)) {
                // TODO preload images
                // TODO is poop
                artist.release_groups.iter()
                    .par_bridge()
                    .flatten()
                    .for_each(|f| {
                        let ent = LibraryEntity::ReleaseGroup(f.clone());
                        thumbnail(&librarian, &ent, 200, 200);
                        thumbnail(&librarian, &ent, 275, 275);
                    });

                ui.upgrade_in_event_loop(move |ui| {
                    ui.set_artist_details((librarian.as_ref(), artist).into());
                    ui.set_page(1)
                }).unwrap();
            }
        });
    }

    fn release_group_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_string();
        let ui = ui.clone();
        std::thread::spawn(move || {
            // TODO starting with this one, figure out the pattern for these functions
            // I need to split this URL better, which means I should probably pass
            // a parsed URL, and I need to get image loading and any significant
            // processing into this thread and not the UI one.
            // For image loading, I need to load the images in this thread and
            // get the thumbs, and only pass the thumbs
            let mbid = url.split_at("dimple://release-group/".len()).1;
            let query = DimpleReleaseGroup { id: mbid.to_string(), ..Default::default() };
            if let Some(LibraryEntity::ReleaseGroup(rel)) = librarian.fetch(&LibraryEntity::ReleaseGroup(query)) {
                ui.upgrade_in_event_loop(move |ui| {
                    ui.set_release_group_details((librarian.as_ref(), rel).into());
                    ui.set_page(2)
                }).unwrap();
            }
        });
    }

    fn release_details(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_string();
        let ui = ui.clone();
        std::thread::spawn(move || {
            let mbid = url.split_at("dimple://release/".len()).1;
            let query = DimpleRelease { id: mbid.to_string(), ..Default::default() };
            if let Some(LibraryEntity::Release(rel)) = librarian.fetch(&LibraryEntity::Release(query)) {
                ui.upgrade_in_event_loop(move |ui| {
                    ui.set_release_details((librarian.as_ref(), rel).into());
                    ui.set_page(3)
                }).unwrap();
            }
        });
    }
}

impl From<(&Librarian, DimpleArtist)> for ArtistDetailsModel {
    fn from((lib, value): (&Librarian, DimpleArtist)) -> Self {
        let genres: Vec<Link> = value.genres
            .iter()
            .flatten()
            .map(|genre| Link {
                name: genre.name.clone().into(),
                url: format!("dimple://genre/{}", genre.name).into(),
            })
            .collect();

        // Preload images
        // TODO is trash
        value.release_groups
            .iter()
            .flatten()
            .for_each(|f| {
                lib.image(&LibraryEntity::ReleaseGroup(f.clone()));
            });

        // TODO sort
        let mut albums: Vec<_> = value.release_groups
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter(|rel| rel.primary_type.to_lowercase() == "album")
            .collect();
        albums.sort_by_key(|f| f.first_release_date.to_string());
        albums.reverse();
        let albums: Vec<_> = albums.iter()
            .map(|rel| (lib, rel.clone()).into())
            .collect();

        let singles: Vec<CardModel> = value.release_groups
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter(|rel| rel.primary_type.to_lowercase() == "single")
            .map(|rel| (lib, rel.clone()).into())
            .collect();

        let eps: Vec<CardModel> = value.release_groups
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter(|rel| rel.primary_type.to_lowercase() == "ep")
            .map(|rel| (lib, rel.clone()).into())
            .collect();

        let others: Vec<CardModel> = value.release_groups
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter(|rel| rel.primary_type.to_lowercase() != "album" && rel.primary_type.to_lowercase() != "single" && rel.primary_type.to_lowercase() != "ep")
            .map(|rel| (lib, rel.clone()).into())
            .collect();

        // TODO raw links are temporary
        let links: Vec<Link> = value.relations
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter_map(|rel| match rel.content {
                DimpleRelationContent::Url(url) => Some(url),
                _ => None,
            })
            .map(|url| Link {
                name: url.resource.clone().into(),
                url: url.resource.clone().into(),
            })
            .chain(std::iter::once(Link { 
                name: format!("https://musicbrainz.org/artist/{}", value.id).into(),
                url: format!("https://musicbrainz.org/artist/{}", value.id).into(),
            }))
            .collect();
        // TODO think I'm not supposed to create new models every time. Not sure
        // where I get their type then, like for VecModel.
        // https://slint.dev/releases/1.3.2/docs/rust/slint/struct.SortModel
        ArtistDetailsModel {
            disambiguation: value.disambiguation.clone().into(),
            summary: value.summary.clone().map(|b| b.value).unwrap_or("".to_string()).into(),
            // TODO get rid of the card and pass the image(s) in higher res
            card: (lib, value).into(), 
            genres: ModelRc::from(genres.as_slice()),
            albums: ModelRc::from(albums.as_slice()),
            singles: ModelRc::from(singles.as_slice()),
            eps: ModelRc::from(eps.as_slice()),
            others: ModelRc::from(others.as_slice()),
            links: ModelRc::from(links.as_slice()),
        }
    }
}

impl From<(&Librarian, DimpleReleaseGroup)> for ReleaseGroupDetailsModel {
    fn from((lib, value): (&Librarian, DimpleReleaseGroup)) -> Self {
        let genres: Vec<Link> = value.genres
            .iter()
            .flatten()
            .map(|genre| Link {
                name: genre.name.clone().into(),
                url: format!("dimple://genre/{}", genre.name).into(),
            })
            .collect();

        let artists: Vec<Link> = value.artists
            .iter()
            .flatten()
            .map(|artist| Link {
                name: artist.name.clone().into(),
                url: format!("dimple://artist/{}", artist.id).into(),
            })
            .collect();

        // Preload images
        // TODO is trash
        // value.releases
        //     .par_iter()
        //     .flatten()
        //     .for_each(|f| {
        //         lib.image(&LibraryEntity::ReleaseGroup(f.clone()));
        //     });

        // TODO sort
        let releases: Vec<CardModel> = value.releases
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            // .filter(|rel| rel.primary_type == "Album")
            .map(|rel| (lib, rel.clone()).into())
            .collect();


        // TODO raw links are temporary
        let links: Vec<Link> = value.relations
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter_map(|rel| match rel.content {
                DimpleRelationContent::Url(url) => Some(url),
                _ => None,
            })
            .map(|url| Link {
                name: url.resource.clone().into(),
                url: url.resource.clone().into(),
            })
            .chain(std::iter::once(Link { 
                name: format!("https://musicbrainz.org/release-group/{}", value.id).into(),
                url: format!("https://musicbrainz.org/release-group/{}", value.id).into(),
            }))
            .collect();
        ReleaseGroupDetailsModel {
            disambiguation: value.disambiguation.clone().into(),
            summary: value.summary.clone().map(|b| b.value).unwrap_or("".to_string()).into(),
            // TODO get rid of the card and pass the image(s) in higher res
            card: (lib, value.clone()).into(), 
            genres: ModelRc::from(genres.as_slice()),
            releases: ModelRc::from(releases.as_slice()),
            links: ModelRc::from(links.as_slice()),
            primary_type: value.primary_type.clone().into(),
            artists: ModelRc::from(artists.as_slice()),
            media: ModelRc::default(),
        }
    }
}

impl From<(&Librarian, DimpleRelease)> for ReleaseDetailsModel {
    fn from((lib, value): (&Librarian, DimpleRelease)) -> Self {
        let genres: Vec<Link> = value.genres
            .iter()
            .flatten()
            .map(|genre| Link {
                name: genre.name.clone().into(),
                url: format!("dimple://genre/{}", genre.name).into(),
            })
            .collect();

        let artists: Vec<Link> = value.artists
            .iter()
            .flatten()
            .map(|artist| Link {
                name: artist.name.clone().into(),
                url: format!("dimple://artist/{}", artist.id).into(),
            })
            .collect();

        // TODO raw links are temporary
        let links: Vec<Link> = value.relations
            .iter()
            .flatten()
            .map(|rel| rel.to_owned())
            .filter_map(|rel| match rel.content {
                DimpleRelationContent::Url(url) => Some(url),
                _ => None,
            })
            .map(|url| Link {
                name: url.resource.clone().into(),
                url: url.resource.clone().into(),
            })
            .chain(std::iter::once(Link { 
                name: format!("https://musicbrainz.org/release/{}", value.id).into(),
                url: format!("https://musicbrainz.org/release/{}", value.id).into(),
            }))
            .collect();
        ReleaseDetailsModel {
            disambiguation: value.disambiguation.clone().into(),
            summary: value.summary.clone().map(|b| b.value).unwrap_or("".to_string()).into(),
            // TODO get rid of the card and pass the image(s) in higher res
            card: (lib, value.clone()).into(), 
            genres: ModelRc::from(genres.as_slice()),
            links: ModelRc::from(links.as_slice()),
            artists: ModelRc::from(artists.as_slice()),
            release_group: Link { 
                name: value.release_group.clone().map(|f| f.title).unwrap_or("".to_string()).into(),
                url: format!("dimple://release-group/{}", 
                    value.release_group.map(|f| f.id).unwrap_or("".to_string())).into(),
            },
        }
    }
}

impl From<(&Librarian, LibraryEntity)> for CardModel {
    fn from((librarian, value): (&Librarian, LibraryEntity)) -> Self {
        match value {
            LibraryEntity::Artist(artist) => (librarian, artist).into(),
            LibraryEntity::ReleaseGroup(release_group) => (librarian, release_group).into(),
            LibraryEntity::Release(release) => (librarian, release).into(),
            LibraryEntity::Genre(genre) => genre.into(),
            LibraryEntity::Track(track) => track.into(),
        }
    }
}

impl From<(&Librarian, DimpleArtist)> for CardModel {
    fn from((lib, artist): (&Librarian, DimpleArtist)) -> Self {
        let ent = LibraryEntity::Artist(artist.clone());
        CardModel {
            title: Link { 
                name: artist.name.clone().into(), 
                url: format!("dimple://artist/{}", &artist.id).into() 
            },
            sub_title: [
                Link { 
                    name: artist.disambiguation.clone().into(), 
                    url: "".into() 
                }
            ].into(),
            image: ImageLink { 
                image: thumbnail(lib, &ent, 200, 200), 
                name: artist.name.clone().into(), 
                url: format!("dimple://artist/{}", &artist.id).into() 
            },
        }
    }
}

impl From<(&Librarian, DimpleReleaseGroup)> for CardModel {
    fn from((lib, release_group): (&Librarian, DimpleReleaseGroup)) -> Self {
        let ent = LibraryEntity::ReleaseGroup(release_group.clone());
        CardModel {
            title: Link { 
                name: release_group.title.clone().into(), 
                url: format!("dimple://release-group/{}", release_group.id.clone()).into() 
            },
            sub_title: [
                Link { 
                    name: format!("{:.4} {}", release_group.first_release_date, release_group.primary_type).into(),
                    url: "".into() 
                }
            ].into(),
            image: ImageLink { 
                image: thumbnail(lib, &ent, 200, 200), 
                name: release_group.title.clone().into(), 
                url: format!("dimple://release-group/{}", release_group.id.clone()).into() 
            },
        }
    }
}

impl From<(&Librarian, DimpleRelease)> for CardModel {
    fn from((lib, release): (&Librarian, DimpleRelease)) -> Self {
        let ent = LibraryEntity::Release(release.clone());
        // TODO 
        // let format = release.media[0].format
        let format = "CD";
        CardModel {
            title: Link { 
                name: release.title.clone().into(), 
                url: format!("dimple://release/{}", release.id.clone()).into() 
            },
            sub_title: [
                Link { 
                    name: format!("{} {} {}", format, release.country, release.date).into(),
                    url: "".into() 
                }
            ].into(),
            image: ImageLink { 
                image: thumbnail(lib, &ent, 200, 200), 
                name: release.title.clone().into(), 
                url: format!("dimple://release/{}", release.id.clone()).into() 
            },
        }
    }
}

impl From<DimpleTrack> for CardModel {
    fn from(_track: DimpleTrack) -> Self {
        CardModel::default()
    }
}

impl From<DimpleGenre> for CardModel {
    fn from(genre: DimpleGenre) -> Self {
        let dynamic_image = DynamicImage::default();
        let slint_image = dynamic_image_to_slint_image(&dynamic_image);
        CardModel {
            title: Link { name: genre.name.clone().into(), url: genre.name.clone().into() },
            sub_title: [Link { name: "".into(), url: "".into() }].into(),
            image: ImageLink { image: slint_image, name: genre.name.clone().into(), url: genre.name.clone().into() },
        }
    }
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

/// Get a thumbnail for the given entity as a Slint image. 
/// TODO future improvement will cache the Slint  image or the SharedPixBuf,
/// whichever is thread safe. The goal is to stop copying the images.
/// Okay, since it seems like this has to ilve in the event loop, we might
/// be able to return a default and then ping the UI when it's loaded.
pub fn thumbnail(library: &Librarian, entity: &LibraryEntity, width: u32, 
    height: u32) -> slint::Image {

    library.thumbnail(entity, width, height)
        .or_else(|| Some(DynamicImage::default()))
        .map(|dyn_image| dynamic_image_to_slint_image(&dyn_image))
        .unwrap()
}
