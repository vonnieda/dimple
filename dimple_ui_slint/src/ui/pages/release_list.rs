use anyhow::Result;
use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::DimpleEntity;
use dimple_core::model::Release;
use dimple_db::db::query::QuerySubscription;
use slint::ComponentHandle;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseListAdapter;
use slint::Model as _;

pub struct ReleaseListController {
    _releases_subscription: QuerySubscription,
}

impl ReleaseListController {
    pub fn new(app: &App) -> Result<Self> {
        // Subscribe to release changes
        let ui = app.ui.clone();
        let images = app.images.clone();
        let library = app.library.clone();
        let releases_subscription = app.library.db.query_subscribe(
            "SELECT * FROM Release ORDER BY title ASC",
            (),
            move |releases: Vec<Release>| {
                log::info!("Releases refreshed: {} releases", releases.len());
                let images = images.clone();
                let library = library.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let cards = release_cards(&images, &releases, &library);
                    let adapter = ui.global::<ReleaseListAdapter>();
                    adapter.set_cards(ModelRc::from(cards.as_slice()));
                }).unwrap();
            },
        )?;

        Ok(Self {
            _releases_subscription: releases_subscription,
        })
    }
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(&DimpleEntity::from(&release), 200, 200, move |ui, image| {
                let adapter = ui.global::<ReleaseListAdapter>();
                let mut card = adapter.get_cards().row_data(index).unwrap();
                card.image.image = image;
                adapter.get_cards().set_row_data(index, card);
            });
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

