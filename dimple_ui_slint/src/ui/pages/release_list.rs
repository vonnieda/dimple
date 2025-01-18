use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Release;
use dimple_core::model::ModelBasics;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use slint::Model as _;

pub fn release_list_init(app: &App) {
    let app = app.clone();
    let library = app.library.clone();
    // library.on_change(Box::new(move |_event| update_model(&app)));
}

pub fn release_list(app: &App) {
    update_model(app);
    app.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::ReleaseList)).unwrap();
}

fn update_model(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let library = app.library.clone();
        let releases = library.query("
            SELECT * 
            FROM Release
            ORDER BY title ASC
        ", ());
        let ui = app.ui.clone();
        let images = app.images.clone();
        ui.upgrade_in_event_loop(move |ui| {
            let cards = release_cards(&images, &releases, &library);
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_release_list(adapter);
        }).unwrap();
    });
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

