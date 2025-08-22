use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardSectionAdapter;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use slint::ComponentHandle as _;
use slint::ModelRc;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::GenreDetailsAdapter;
use crate::ui::ImageLinkAdapter;
use slint::Model as _;

pub fn genre_details_init(app: &App) {
    let app1 = app.clone();

    // TODO filter events by key - but we can't get the key without the
    // UI, so rethink the whole mess.
    let app1 = app.clone();
    app.library.notifier.observe(move |event| if event.type_name == "Genre" { update_model(&app1) });
}

pub fn genre_details(url: &str, app: &App) {
    let app = app.clone();
    let url = Url::parse(url).unwrap();
    let key = url.path_segments().unwrap().next().unwrap().to_string();
    let ui = app.ui.clone();
    ui.upgrade_in_event_loop(move |ui| {
        ui.global::<GenreDetailsAdapter>().set_key(key.into());
        update_model(&app);
        ui.set_page(Page::GenreDetails);
    }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<GenreDetailsAdapter>().get_key();
        if key.is_empty() {
            return
        }
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let genre = Genre::get(&library, &key).unwrap();
            let links = genre.links(&library);
            let releases = genre.releases(&app.library);
            let artists = genre.artists(&app.library);
            let genres = vec![genre.clone()];
            let ui = app.ui.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let card: CardAdapter = genre.clone().into();                
                let links: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                        url: link.url.clone().into(),
                    }
                })
                .collect();

                let mut sections: Vec<CardSectionAdapter> = vec![];

                if !artists.is_empty() {
                    sections.push(CardSectionAdapter {
                        title: "Artists".into(),
                        sub_title: Default::default(),
                        cards: artist_cards(&artists).as_slice().into(),
                        ..Default::default()
                    });
                }
    
                if !releases.is_empty() {
                    sections.push(CardSectionAdapter {
                        title: "Releases".into(),
                        sub_title: Default::default(),
                        cards: release_cards(&releases, &app.library).as_slice().into(),
                        ..Default::default()
                    });
                }
    
                if !genres.is_empty() {
                    sections.push(CardSectionAdapter {
                        title: "Related Genres".into(),
                        sub_title: Default::default(),
                        cards: genre_cards(&genres).as_slice().into(),
                        ..Default::default()
                    });
                }
    
                ui.global::<GenreDetailsAdapter>().set_card(card);
                ui.global::<GenreDetailsAdapter>().set_key(genre.id.clone().unwrap_or_default().into());
                ui.global::<GenreDetailsAdapter>().set_summary(genre.summary.clone().unwrap_or_default().into());
                ui.global::<GenreDetailsAdapter>().set_disambiguation(genre.disambiguation.clone().unwrap_or_default().into());
                ui.global::<GenreDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<GenreDetailsAdapter>().set_dump(format!("{genre:?}").into());
                ui.global::<GenreDetailsAdapter>().set_sections(sections.as_slice().into());
            }).unwrap();
        });
    }).unwrap();
}

fn release_cards(releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.id.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn artist_cards(artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let card: CardAdapter = artist_card(&artist);
            card
        })
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.id.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn genre_cards(genres: &[Genre]) -> Vec<CardAdapter> {
    genres.iter().cloned().enumerate()
        .map(|(index, genre)| {
            let card: CardAdapter = genre_card(&genre);
            card
        })
        .collect()
}

fn genre_card(genre: &Genre) -> CardAdapter {
    let genre = genre.clone();
    CardAdapter {
        key: genre.id.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: genre.disambiguation.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        },
    }
}

