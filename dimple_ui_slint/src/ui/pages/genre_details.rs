use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
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
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app1.clone();
        ui.global::<GenreDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = app1.clone();
        ui.global::<GenreDetailsAdapter>().on_play_next(move |key| play_next(&app, &key));
        let app = app1.clone();
        ui.global::<GenreDetailsAdapter>().on_play_later(move |key| play_later(&app, &key));
    }).unwrap();

    // TODO filter events by key - but we can't get the key without the
    // UI, so rethink the whole mess.
    let app1 = app.clone();
    app.library.on_change(Box::new(move |event| if event.type_name == "Genre" { update_model(&app1) }));
}

pub fn genre_details(url: &str, app: &App) {
    let app = app.clone();
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();
    let ui = app.ui.clone();
    ui.upgrade_in_event_loop(move |ui| {
        ui.global::<GenreDetailsAdapter>().set_key(key.into());
        update_model(&app);
        ui.set_page(Page::GenreDetails);
    }).unwrap();
}

fn play_now(app: &App, key: &str) {
    app.player.play_now(&Genre::get(&app.library, key).unwrap());
}

fn play_next(app: &App, key: &str) {
    app.player.play_next(&Genre::get(&app.library, key).unwrap());
}

fn play_later(app: &App, key: &str) {
    app.player.play_later(&Genre::get(&app.library, key).unwrap());
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
            let ui = app.ui.clone();
            let images = app.images.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut card: CardAdapter = genre.clone().into();                
                card.image.image = app.images.lazy_get(genre.clone(), 275, 275, |ui, image| {
                    let mut card = ui.global::<GenreDetailsAdapter>().get_card();
                    card.image.image = image;
                    ui.global::<GenreDetailsAdapter>().set_card(card);
                });

                let links: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                        url: link.url.clone().into(),
                    }
                })
                .collect();

                let releases = release_cards(&images, &releases, &library);
                ui.global::<GenreDetailsAdapter>().set_card(card.into());
                ui.global::<GenreDetailsAdapter>().set_key(genre.key.clone().unwrap_or_default().into());
                ui.global::<GenreDetailsAdapter>().set_releases(ModelRc::from(releases.as_slice()));
                ui.global::<GenreDetailsAdapter>().set_summary(genre.summary.clone().unwrap_or_default().into());
                ui.global::<GenreDetailsAdapter>().set_disambiguation(genre.disambiguation.clone().unwrap_or_default().into());
                ui.global::<GenreDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<GenreDetailsAdapter>().set_dump(format!("{:?}", genre).into());
            }).unwrap();
        });
    }).unwrap();
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(release.clone(), 200, 200, move |ui, image| {
                let mut card = ui.get_release_list().cards.row_data(index).unwrap();
                card.image.image = image;
                ui.get_release_list().cards.set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
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
