use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use anyhow::Result;
use dimple_core::model::Artist;
use dimple_core::model::DimpleEntity;
use dimple_db::db::query::QuerySubscription;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use slint::Model as _;

pub struct ArtistListController {
    _artists_subscription: QuerySubscription,
}

impl ArtistListController {
    pub fn new(app: &App) -> Result<Self> {
        let sql = "
            SELECT * 
            FROM Artist
            ORDER BY lower(name) ASC, lower(disambiguation) ASC
        ";
        let images = app.images.clone();
        let ui = app.ui.clone();
        let artists_subscription = app.library.db.query_subscribe(sql, (), move |artists| {
            let images = images.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let cards = artist_cards(&images, &artists);
                let adapter = CardGridAdapter {
                    cards: ModelRc::from(cards.as_slice()),
                };
                ui.set_artist_list(adapter);
            }).unwrap();
        })?;
        
        Ok(Self {
            _artists_subscription: artists_subscription,
        })
    }
}

fn artist_cards(images: &ImageMangler, artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let mut card: CardAdapter = artist_card(&artist);
            card.image.image = images.lazy_get(&DimpleEntity::from(&artist), 200, 200, move |ui, image| {
                let mut card = ui.get_artist_list().cards.row_data(index).unwrap();
                card.image.image = image;
                ui.get_artist_list().cards.set_row_data(index, card);
            });
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

