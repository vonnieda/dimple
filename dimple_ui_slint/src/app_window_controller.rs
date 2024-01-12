use dimple_musicbrainz_library::MusicBrainzLibrary;
use dimple_lastfm_library::LastFmLibrary;
use dimple_fanart_tv_library::FanartTvLibrary;
use dimple_deezer_library::DeezerLibrary;

use std::sync::Arc;

use dimple_core::{model::{Artist, Genre, Track, Release, MusicbrainzReleaseGroup}, library::{Library, LibraryEntity}};
use dimple_librarian::librarian::Librarian;
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
        
        self.ui.global::<Navigator>().on_navigate(move |url| 
            Self::navigate(url, librarian.clone(), ui.clone()));

        // self.librarian.add_library(Arc::new(FolderLibrary::new("/Users/jason/Music/My Music")));
        self.librarian.add_library(Box::<MusicBrainzLibrary>::default());
        self.librarian.add_library(Box::<LastFmLibrary>::default());
        self.librarian.add_library(Box::<FanartTvLibrary>::default());
        self.librarian.add_library(Box::<DeezerLibrary>::default());

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }

    fn navigate(url: slint::SharedString, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        // dbg!(&url);
        // let url = Url::parse(&url);
        if url.starts_with("dimple://home") {
            Self::home(ui);
        } 
        else if url.starts_with("dimple://search") {
            Self::search(&url, librarian, ui);
        }
        else if url == "dimple://artists" {
            Self::artists(librarian, ui);
        }
        else if url.starts_with("dimple://artists/") {
            Self::artist(&url, librarian, ui);
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

    fn artist(url: &str, librarian: LibrarianHandle, ui: slint::Weak<AppWindow>) {
        let url = url.to_string();
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
        let releases: Vec<CardModel> = value.mb.release_groups
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
            // TODO get rid of the card and pass the image(s) in higher res
            card: (lib, value).into(), 
            genres: ModelRc::from(genres.as_slice()),
            releases: ModelRc::from(releases.as_slice()),
        }
    }
}

impl From<MusicbrainzReleaseGroup> for CardModel {
    fn from(value: MusicbrainzReleaseGroup) -> Self {
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
        let ent = LibraryEntity::Release(release.clone());
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
                image: thumbnail(lib, &ent, 500, 500), 
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
        let ent = LibraryEntity::Artist(artist.clone());
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
                image: thumbnail(library, &ent, 500, 500), 
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
