use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::ArtistListAdapter;
use anyhow::Result;
use dimple_core::model::Artist;
use dimple_db::db::query::QuerySubscription;
use slint::ComponentHandle as _;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

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
        let ui = app.ui.clone();
        let artists_subscription = app.library.db.query_subscribe(sql, (), move |artists| {
            ui.upgrade_in_event_loop(move |ui| {
                let cards = artist_cards(&artists);
                let adapter = ui.global::<ArtistListAdapter>();
                adapter.set_cards(ModelRc::from(cards.as_slice()));
            }).unwrap();
        })?;
        
        Ok(Self {
            _artists_subscription: artists_subscription,
        })
    }
}

fn artist_cards(artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned()
        .map(|artist| artist_card(&artist))
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.id.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
            image: Default::default(),
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

