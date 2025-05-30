use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::librarian;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use slint::ComponentHandle as _;
use slint::ModelRc;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ArtistDetailsAdapter;
use crate::ui::ImageLinkAdapter;
use slint::Model as _;

pub fn artist_details_init(app: &App) {
    // TODO filter events by key - but we can't get the key without the
    // UI, so rethink the whole mess.
    let app1 = app.clone();
    app.library.notifier.observe(move |event| {
        // TODO so gross.
        // TODO actually need to make the event include the model so that here
        // we actually check for DimageRef with our key, GenreRef with our key, etc.
        if event.type_name == "Artist" || event.type_name == "Release" || event.type_name == "Link" || event.type_name == "Genre" { 
            update_model(&app1);
        }
    });
}

pub fn artist_details(url: &str, app: &App) {
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();

    let app1 = app.clone();
    let key1 = key.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<ArtistDetailsAdapter>().set_key(key.into());
        update_model(&app1);
        ui.set_page(Page::ArtistDetails);

        let app2 = app1.clone();
        let key2 = key1.clone();
        std::thread::spawn(move || {
            if let Some(artist) = Artist::get(&app2.library, &key2) {
                librarian::refresh_metadata(&app2.library, &app2.plugins, &artist);
            }
        });    
    }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<ArtistDetailsAdapter>().get_key();
        if key.is_empty() {
            return
        }
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let artist = Artist::get(&library, &key).unwrap();
            let genres = artist.genres(&library);
            let links = artist.links(&library);
            let releases = artist.releases(&app.library);
            let ui = app.ui.clone();
            let images = app.images.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut card: CardAdapter = artist.clone().into();                
                card.image.image = app.images.lazy_get(artist.clone(), 275, 275, |ui, image| {
                    let mut card = ui.global::<ArtistDetailsAdapter>().get_card();
                    card.image.image = image;
                    ui.global::<ArtistDetailsAdapter>().set_card(card);
                });

                let genres = genre_links(&genres);
                let links: Vec<LinkAdapter> = links.iter().map(|link| {
                        LinkAdapter {
                            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                            url: link.url.clone().into(),
                        }
                    })
                    .collect();

                let releases = release_cards(&images, &releases);
                ui.global::<ArtistDetailsAdapter>().set_card(card.into());
                ui.global::<ArtistDetailsAdapter>().set_key(artist.key.clone().unwrap_or_default().into());
                ui.global::<ArtistDetailsAdapter>().set_releases(ModelRc::from(releases.as_slice()));
                ui.global::<ArtistDetailsAdapter>().set_summary(artist.summary.clone().unwrap_or_default().into());
                ui.global::<ArtistDetailsAdapter>().set_disambiguation(artist.disambiguation.clone().unwrap_or_default().into());
                ui.global::<ArtistDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
                ui.global::<ArtistDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<ArtistDetailsAdapter>().set_dump(format!("{:?}", artist).into());
            }).unwrap();
        });
    }).unwrap();
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn release_cards(images: &ImageMangler, releases: &[Release]) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release);
            card.image.image = images.lazy_get(release.clone(), 200, 200, move |ui, image| {
                let adapter = ui.global::<ArtistDetailsAdapter>();
                let mut card = adapter.get_releases().row_data(index).unwrap();
                card.image.image = image;
                adapter.get_releases().set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn release_card(release: &Release) -> CardAdapter {
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
            name: format!("{} {}", 
                release.date.unwrap_or_default(), 
                release.release_group_type.unwrap_or_default()).into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

