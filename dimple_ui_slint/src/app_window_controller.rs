use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_deezer_library::DeezerLibrary;
use musicbrainz_rs::entity::release_group::ReleaseGroup;

use std::sync::Arc;

use dimple_core::{model::{Artist, Genre, Track, Release}, library::{Library, LibraryEntity}};
use dimple_librarian::librarian::{Librarian, self};
// use dimple_player::player::{Player, PlayerHandle};
use image::DynamicImage;
use slint::{ModelRc, SharedPixelBuffer, Rgba8Pixel, ComponentHandle};

slint::include_modules!();

pub type LibrarianHandle = Arc<Librarian>;

pub struct AppWindowController {
    ui: AppWindow,
    librarian: LibrarianHandle,
    // player: PlayerHandle,
}

impl AppWindowController {
    fn navigate(url: slint::SharedString, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        dbg!(&url);
        if url.starts_with("dimple://home") {
            let ui = ui.clone();
            ui.upgrade_in_event_loop(move |ui| {
                ui.set_card_grid_cards(ModelRc::from(vec![].as_slice()));
                ui.set_page(0)
            }).unwrap();
        } 
        else if url.starts_with("dimple://search") {
            let ui = ui.clone();
            std::thread::spawn(move || {
                let query_str = url.split_at("dimple://search".len()).1;
                let search_results: Vec<LibraryEntity> = librarian.search(query_str).collect();
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
        else if url == "dimple://artists" {
            let ui = ui.clone();
            std::thread::spawn(move || {
                let mut artists: Vec<Artist> = librarian.artists().collect();
                artists.sort_by_key(|a| a.name().to_lowercase());
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
        else if url.starts_with("dimple://artists/") {
            let ui = ui.clone();
            std::thread::spawn(move || {
                // TODO ew
                let mbid = url.split_at("dimple://artists/".len()).1;
                let query = dimple_core::model::Artist::with_mbid(mbid);
                if let Some(LibraryEntity::Artist(artist)) = librarian.fetch(&LibraryEntity::Artist(query)) {
                    ui.upgrade_in_event_loop(move |ui| {
                        ui.set_artist_details((librarian.as_ref(), artist).into());
                        ui.set_page(1)
                    }).unwrap();
                }
            });
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        
        self.ui.global::<Navigator>().on_navigate(move |url| 
            Self::navigate(url, librarian.clone(), ui.clone()));

        // self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music")));
        self.librarian.add_library(Box::<MusicBrainzLibrary>::default());
        self.librarian.add_library(Box::<FanartTvLibrary>::default());
        // self.librarian.add_library(Box::<LastFmLibrary>::default());
        // self.librarian.add_library(Box::<DeezerLibrary>::default());

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }
}

impl From<(&Librarian, Artist)> for ArtistDetailsModel {
    fn from((lib, value): (&Librarian, Artist)) -> Self {
        let genres: Vec<Link> = value.mb.genres
            .iter()
            .flatten()
            .map(|genre| Link {
                name: genre.name.clone().into(),
                ..Default::default()
            })
            .collect();
        // TODO this should be release groups, but they aren't serializing for
        // some reason.
        let releases: Vec<CardModel> = value.mb.releases
            .iter()
            .flatten()
            .map(|rel| Release {
                mb: rel.clone(),
            })
            .map(|rel| (lib, rel.clone()).into())
            .collect();
        ArtistDetailsModel {
            disambiguation: value.mb.disambiguation.clone().into(),
            bio: "".to_string().into(), 
            card: (lib, value).into(), 
            genres: ModelRc::from(genres.as_slice()),
            releases: ModelRc::from(releases.as_slice()),
        }
    }
}

impl From<ReleaseGroup> for CardModel {
    fn from(value: ReleaseGroup) -> Self {
        CardModel {
            title: Link {
                name: value.title.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl From<(&Librarian, Release)> for CardModel {
    fn from((lib, release): (&Librarian, Release)) -> Self {
        let slint_image = lib.image(&LibraryEntity::Release(release.clone()))
            .or_else(|| Some(DynamicImage::default()))
            .map(|dyn_image| dynamic_image_to_slint_image(&dyn_image))
            .unwrap();
        CardModel {
            title: Link { 
                name: release.mb.title.clone().into(), 
                url: format!("dimple://releases/{}", release.mbid()).into() 
            },
            sub_title: [
                Link { 
                    name: "".into(), 
                    url: "".into() 
                }
            ].into(),
            image: ImageLink { 
                image: slint_image, 
                name: release.title().into(), 
                url: format!("dimple://releases/{}", release.mbid()).into() 
            },
        }
    }
}

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        let librarian = Arc::new(Librarian::default());
        // let player = Player::new(librarian.clone());
        Self {
            ui,
            librarian,
            // player,
        }
    }
}

impl From<(&Librarian, LibraryEntity)> for CardModel {
    fn from((librarian, value): (&Librarian, LibraryEntity)) -> Self {
        match value {
            LibraryEntity::Artist(artist) => (librarian, artist).into(),
            LibraryEntity::Genre(genre) => genre.into(),
            LibraryEntity::Track(track) => track.into(),
            LibraryEntity::Release(release) => release.into(),
        }
    }
}

impl From<(&Librarian, Artist)> for CardModel {
    fn from((library, artist): (&Librarian, Artist)) -> Self {
        // TODO this needs cached locally
        let slint_image = library.image(&LibraryEntity::Artist(artist.clone()))
            .or_else(|| Some(DynamicImage::default()))
            .map(|dyn_image| dynamic_image_to_slint_image(&dyn_image))
            .unwrap();
        CardModel {
            title: Link { 
                name: artist.name().into(), 
                url: format!("dimple://artists/{}", artist.mbid()).into() 
            },
            sub_title: [
                Link { 
                    name: artist.mb.disambiguation.clone().into(), 
                    url: "".into() 
                }
            ].into(),
            image: ImageLink { 
                image: slint_image, 
                name: artist.name().into(), 
                url: format!("dimple://artists/{}", artist.mbid()).into() 
            },
        }
    }
}

impl From<Track> for CardModel {
    fn from(_track: Track) -> Self {
        CardModel::default()
    }
}

impl From<Release> for CardModel {
    fn from(_release: Release) -> Self {
        CardModel::default()
    }
}

impl From<Genre> for CardModel {
    fn from(genre: Genre) -> Self {
        let dynamic_image = DynamicImage::default();
        let slint_image = dynamic_image_to_slint_image(&dynamic_image);
        CardModel {
            title: Link { name: genre.name.clone().into(), url: genre.url.clone().into() },
            sub_title: [Link { name: "".into(), url: "".into() }].into(),
            image: ImageLink { image: slint_image, name: genre.name.clone().into(), url: genre.url.clone().into() },
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

