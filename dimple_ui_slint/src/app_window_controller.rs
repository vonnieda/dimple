use std::{sync::Arc, time::Duration};

use dimple_core::{library::{LibraryHandle}, model::{Artist, Release, Genre}};
use dimple_librarian::librarian::Librarian;
use dimple_player::player::{Player, PlayerHandle};
use image::DynamicImage;
use slint::{ModelRc, Weak, SharedPixelBuffer, Rgba8Pixel};
use dimple_folder_library::folder_library::FolderLibrary;

slint::include_modules!();

pub type LibrarianHandle = Arc<Librarian>;

pub struct AppWindowController {
    ui: AppWindow,
    librarian: LibrarianHandle,
    player: PlayerHandle,
}

impl AppWindowController {
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        let library: LibraryHandle = librarian.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| {
            dbg!(&url);
            let ui = ui.unwrap();
            if url.starts_with("dimple://home") {
                todo!()
            } 
            else if url.starts_with("dimple://artists/") {
                let artist_url = url.split_at(17).1;

                let artist = library.artists()
                    .into_iter()
                    .find(|a| a.url == artist_url)
                    .unwrap();
                    
                ui.as_weak().upgrade_in_event_loop(move |ui| { 
                    // let mut card_models: Vec<CardModel> = vec![];
                    // for item in vec {
                    //     card_models.push(item.into());
                    // }
                    // ui.set_card_grid_model(CardGridModel { 
                    //     cards: ModelRc::from(card_models.as_slice()) 
                    // }); 
                    ui.set_artist_details_model(ArtistDetailsModel {
                        card: artist.clone().into(),
                        genres: ModelRc::from(artist.genres
                            .iter()
                            .map(|genre| Link { name: genre.name.clone().into(), url: genre.url.clone().into() })
                            .collect::<Vec<Link>>()
                            .as_slice()),
                        releases: Default::default(),
                        bio: "Formed in 1932 during the bombing of the Mars Luna Expressway, the six man band, which very quickly became a duo, came to define the genre Snake Punk. After Alexander recovered from the the battle, the two wrote 'Vaporize Your Expressway', which some saw as a response to the unfortunate but unpreventable incident at Mars Luna. Regardless, it was an instant classic and blasted the band to the top of the galactic charts.".into(),
                    });
                    ui.set_page(1);                
                })
                .unwrap();
            }
            else if url.starts_with("dimple://artists") {
                let mut results: Vec<Artist> = library.artists();
                results.sort_by_key(|artist| artist.name.clone());
                Self::set_card_grid_model(results, ui.as_weak());
                ui.set_page(0);
            } 
            else if url.starts_with("dimple://releases") {
                let mut results: Vec<Release> = library.releases().iter().collect();
                results.sort_by_key(|release| release.title.clone());
                // TODO wow.
                let results: Vec<(LibraryHandle, Release)> = results.into_iter()
                    .map(|release| (library.clone(), release))
                    .collect();
                Self::set_card_grid_model(results, ui.as_weak());
                ui.set_page(0);
            } 
            else if url.starts_with("dimple://genres") {
                let results: Vec<Genre> = library.genres();
                Self::set_card_grid_model(results, ui.as_weak());
                ui.set_page(0);
            } 
            else if url.starts_with("dimple://search") {
                ui.set_page(4);
            }
        });

        self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music/Opeth")));
        self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music/The Mars Volta")));
        self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music/Metallica")));
        let librarian = self.librarian.clone();
        // TODO gonna change this so the librarian is threaded and just manages its
        // own state.
        // Need to think about folder monitoring, and remote library monitoring
        // and so on.
        std::thread::spawn(move || {
            loop {
                librarian.refresh_all_libraries();
                // TODO think about how to refresh - one way is to navigate to
                // the current page, but that's gonna fuck the scroll position.
                std::thread::sleep(Duration::from_secs(10));
            }
        });

        self.ui.global::<Navigator>().invoke_navigate("dimple://artists".into());

        self.ui.run()
    }

    fn set_card_grid_model<U>(vec: Vec<U>, ui: Weak<AppWindow>) 
        where U: Into<CardModel> + Send + 'static,
    {
        ui.upgrade_in_event_loop(move |ui| { 
            let mut card_models: Vec<CardModel> = vec![];
            for item in vec {
                card_models.push(item.into());
            }
            ui.set_card_grid_model(CardGridModel { 
                cards: ModelRc::from(card_models.as_slice()) 
            }); 
        })
        .unwrap();

    }
}

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        let librarian = Arc::new(Librarian::default());
        let player = Player::new(librarian.clone());
        Self {
            ui,
            librarian,
            player,
        }
    }
}

impl From<(LibraryHandle, Release)> for CardModel {
    fn from((library, release): (LibraryHandle, Release)) -> Self {
        let slint_image = release.art.first()
            .and_then(|image| library.image(image).ok())
            .or_else(|| Some(DynamicImage::default()))
            // TODO magic
            .map(|dynamic_image| dynamic_image.resize(500, 500, image::imageops::FilterType::Nearest))
            .map(|dynamic_image| dynamic_image_to_slint_image(&dynamic_image))
            .unwrap();

        let artist_url = match release.artists.first() {
            Some(artist) => {
                artist.url.to_string()
            },
            None =>  {
                "".to_string()
            }
        };
        CardModel {
            title: Link { 
                name: release.title.clone().into(), 
                url: release.url.clone().into() 
            },
            sub_title: [
                Link { 
                    name: release.artist().into(), 
                    url: format!("dimple://artists/{}", release.artist()).into() 
                }
            ].into(),
            image: ImageLink { 
                image: slint_image, 
                name: release.title.clone().into(), 
                url: release.url.clone().into() 
            },
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
                url: format!("dimple://artists/{}", artist.url).into() 
            },
            sub_title: [Link { name: "".into(), url: "".into() }].into(),
            image: ImageLink { 
                image: slint_image, 
                name: artist.name.clone().into(), 
                url: format!("dimple://artists/{}", artist.url).into() 
            },
        }
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

