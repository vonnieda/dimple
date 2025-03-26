
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::Track;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardSectionAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::Page;
use crate::ui::SearchResultsAdapter;
use slint::ComponentHandle as _;

pub fn search_results_init(app: &App) {
}

pub fn search_results(url: &str, app: &App) {
    let app = app.clone();
    let url = Url::parse(&url).unwrap();
    let query = url.path_segments().unwrap().next().unwrap();
    let query = percent_encoding::percent_decode_str(query).decode_utf8_lossy().to_string();
    let query = format!("%{}%", query);
    update_model(&app, &query);
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::SearchResults);
    }).unwrap();    
}

fn update_model(app: &App, query: &str) {
    let app = app.clone();
    let query = query.to_string();
    std::thread::spawn(move || {
        let artists = Artist::query(&app.library, 
            "SELECT * FROM Artist WHERE name LIKE ?1 LIMIT 25", (&query,));
        let releases = Release::query(&app.library, 
            "SELECT * FROM Release WHERE title LIKE ?1 LIMIT 25", (&query,));
        let genres = Genre::query(&app.library, 
            "SELECT * FROM Genre WHERE name LIKE ?1 LIMIT 25", (&query,));
        let tracks = Track::query(&app.library, 
            "SELECT * FROM Track WHERE title LIKE ?1 LIMIT 25", (&query,));
                                    
        let app = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let mut sections: Vec<CardSectionAdapter> = vec![];

            if !tracks.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Tracks".into(),
                    sub_title: Default::default(),
                    cards: track_cards(&app.images, &tracks, &app.library).as_slice().into(),
                });
            }

            if !artists.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Artists".into(),
                    sub_title: Default::default(),
                    cards: artist_cards(&app.images, &artists).as_slice().into(),
                });
            }

            if !releases.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Releases".into(),
                    sub_title: Default::default(),
                    cards: release_cards(&app.images, &releases, &app.library).as_slice().into(),
                });
            }

            if !genres.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Genres".into(),
                    sub_title: Default::default(),
                    cards: genre_cards(&app.images, &genres).as_slice().into(),
                });
            }

            let adapter = ui.global::<SearchResultsAdapter>();
            adapter.set_sections(sections.as_slice().into());
        }).unwrap();
    });
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(release.clone(), 200, 200, move |ui, image| {
                // let adapter = ui.global::<HomeAdapter>();
                // let mut card = adapter.get_releases().row_data(index).unwrap();
                // card.image.image = image;
                // adapter.get_releases().set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn artist_cards(images: &ImageMangler, artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let mut card: CardAdapter = artist_card(&artist);
            card.image.image = images.lazy_get(artist.clone(), 200, 200, move |ui, image| {
                // let mut card = ui.get_artist_list().cards.row_data(index).unwrap();
                // card.image.image = image;
                // ui.get_artist_list().cards.set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.key.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn genre_cards(images: &ImageMangler, genres: &[Genre]) -> Vec<CardAdapter> {
    genres.iter().cloned().enumerate()
        .map(|(index, genre)| {
            let mut card: CardAdapter = genre_card(&genre);
            card.image.image = images.lazy_get(genre.clone(), 200, 200, move |ui, image| {
                // let mut card = ui.get_genre_list().cards.row_data(index).unwrap();
                // card.image.image = image;
                // ui.get_genre_list().cards.set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn genre_card(genre: &Genre) -> CardAdapter {
    let genre = genre.clone();
    CardAdapter {
        key: genre.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: genre.disambiguation.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
    }
}

fn track_cards(images: &ImageMangler, tracks: &[Track], library: &Library) -> Vec<CardAdapter> {
    tracks.iter().cloned().enumerate()
        .map(|(index, track)| {
            let mut card: CardAdapter = track_card(&track, &track.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(track.clone(), 200, 200, move |ui, image| {
                // let adapter = ui.global::<HomeAdapter>();
                // let mut card = adapter.get_releases().row_data(index).unwrap();
                // card.image.image = image;
                // adapter.get_releases().set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn track_card(track: &Track, artist: &Artist) -> CardAdapter {
    let track = track.clone();
    CardAdapter {
        key: track.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

