use dimple_musicbrainz_library::musicbrainz_library::MusicBrainzLibrary;

use std::sync::Arc;

use dimple_core::{model::{Artist, Genre, Track, Release}, library::{Library, LibraryEntity}};
use dimple_librarian::librarian::Librarian;
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
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| {
            let librarian = librarian.clone();
            dbg!(&url);
            if url.starts_with("dimple://home") {
                todo!()
            } 
            else if url.starts_with("dimple://search") {
                let url = url.to_string();
                let ui = ui.clone();
                std::thread::spawn(move || {
                    let query_str = url.split_at("dimple://search".len()).1;
                    let search_results: Vec<LibraryEntity> = librarian.search(query_str).collect();
                    ui.upgrade_in_event_loop(move |ui| {
                        let cards: Vec<CardModel> = search_results.into_iter()
                            .map(Into::into)
                            .collect();
                        ui.set_card_grid_cards(ModelRc::from(cards.as_slice()));
                        ui.set_page(0)
                    }).unwrap();
                });
            }
            else if url.starts_with("dimple://artists/") {
                todo!()
            }
            else if url.starts_with("dimple://artists") {
                let ui = ui.clone();
                std::thread::spawn(move || {
                    let artists: Vec<Artist> = librarian.artists().collect();
                    ui.upgrade_in_event_loop(move |ui| {
                        let cards: Vec<CardModel> = artists.into_iter()
                            .map(Into::into)
                            .collect();
                        ui.set_card_grid_cards(ModelRc::from(cards.as_slice()));
                        ui.set_page(0)
                    }).unwrap();
                });
            }
        });

        // self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music")));
        self.librarian.add_library(Box::<MusicBrainzLibrary>::default());

        self.ui.global::<Navigator>().invoke_navigate("dimple://artists".into());

        self.ui.run()
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

impl From<LibraryEntity> for CardModel {
    fn from(value: LibraryEntity) -> Self {
        match value {
            LibraryEntity::Artist(artist) => artist.into(),
            LibraryEntity::Genre(genre) => genre.into(),
            LibraryEntity::Track(track) => track.into(),
            LibraryEntity::Release(release) => release.into(),
        }
    }
}


impl From<Artist> for CardModel {
    fn from(artist: Artist) -> Self {
        // let images = release.art();
        // let image = images.first().unwrap();
        // let dynamic_image = library.image(image).unwrap();
        let dynamic_image = DynamicImage::default();
        let slint_image = dynamic_image_to_slint_image(&dynamic_image);
        CardModel {
            title: Link { 
                name: artist.name.clone().into(), 
                url: format!("dimple://artists/{}", artist.id).into() 
            },
            sub_title: [Link { name: "".into(), url: "".into() }].into(),
            image: ImageLink { 
                image: slint_image, 
                name: artist.name.clone().into(), 
                url: format!("dimple://artists/{}", artist.id).into() 
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
        // let images = release.art();
        // let image = images.first().unwrap();
        // let dynamic_image = library.image(image).unwrap();
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

