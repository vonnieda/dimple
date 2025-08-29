use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::librarian;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::Link;
use slint::ComponentHandle as _;
use slint::ModelRc;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ArtistDetailsAdapter;
use crate::ui::ImageLinkAdapter;
use dimple_db::db::query::QuerySubscription;
use anyhow::Result;
use std::collections::HashMap;

pub struct ArtistDetailsController {
    current_key: MutableStringParam,
    artist_subscription: QuerySubscription,
    genres_subscription: QuerySubscription,
    links_subscription: QuerySubscription,
    releases_subscription: QuerySubscription,
}


impl ArtistDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let current_key = MutableStringParam::new();
        
        // Set up artist subscription
        let sql = "SELECT * FROM Artist WHERE id = ?";
        let ui = app.ui.clone();
        let artist_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |artists: Vec<Artist>| {
            if let Some(artist) = artists.first() {
                let artist = artist.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = artist.clone().into();                
                    ui.global::<ArtistDetailsAdapter>().set_card(card);
                    ui.global::<ArtistDetailsAdapter>().set_key(artist.id.clone().unwrap_or_default().into());
                    ui.global::<ArtistDetailsAdapter>().set_summary(artist.summary.clone().unwrap_or_default().into());
                    ui.global::<ArtistDetailsAdapter>().set_disambiguation(artist.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ArtistDetailsAdapter>().set_dump(serde_json::to_string_pretty(&artist).unwrap().into());
                }).unwrap();
            }
        })?;

        // Set up genres subscription
        let sql = "
            SELECT g.* FROM Genre g
            JOIN GenreRef gr ON g.id = gr.genre_id
            WHERE gr.model_id = ?
            ORDER BY name ASC
        ";
        let ui = app.ui.clone();
        let genres_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |genres: Vec<Genre>| {
            ui.upgrade_in_event_loop(move |ui| {
                let genre_links = genre_links(&genres);
                ui.global::<ArtistDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
            }).unwrap();
        })?;

        // Set up links subscription
        let sql = "
            SELECT l.* FROM Link l
            JOIN LinkRef lr ON l.id = lr.link_id
            WHERE lr.model_id = ?
            ORDER BY name ASC, url ASC
        ";
        let ui = app.ui.clone();
        let links_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |links: Vec<Link>| {
            ui.upgrade_in_event_loop(move |ui| {
                let link_adapters: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                        url: link.url.clone().into(),
                    }
                }).collect();
                ui.global::<ArtistDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        let sql = "
            SELECT DISTINCT Release.* 
            FROM Release 
            JOIN ArtistRef ON ArtistRef.model_id = Release.id
            WHERE ArtistRef.artist_id = ?
            ORDER BY Release.date DESC, Release.title ASC
        ";
        let ui = app.ui.clone();
        let releases_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |releases: Vec<Release>| {
            ui.upgrade_in_event_loop(move |ui| {
                let grouped_releases = group_releases_by_release_group(&releases);
                let release_cards = release_cards(&grouped_releases);
                ui.global::<ArtistDetailsAdapter>().set_releases(ModelRc::from(release_cards.as_slice()));
            }).unwrap();
        })?;

        Ok(Self {
            current_key,
            artist_subscription,
            genres_subscription,
            links_subscription,
            releases_subscription,
        })
    }

    pub fn set_artist(&mut self, key: String, app: &App) -> Result<()> {
        self.current_key.set(&key);
        
        // Refresh all subscriptions
        self.artist_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.releases_subscription.refresh();

        // Trigger metadata refresh in background
        let app_clone = app.clone();
        let key_clone = key.clone();
        std::thread::spawn(move || {
            if let Some(artist) = Artist::get(&app_clone.library, &key_clone) {
                librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &artist.into());
            }
        });

        Ok(())
    }
}

pub fn artist_details(url: &str, app: &App, controller: &mut ArtistDetailsController) {
    let url = Url::parse(url).unwrap();
    let key = url.path_segments().unwrap().next().unwrap().to_string();

    // Set the artist in the controller which will handle all subscriptions
    controller.set_artist(key, app).unwrap();
    
    // Navigate to the artist details page
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::ArtistDetails);
    }).unwrap();
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn release_cards(releases: &[Release]) -> Vec<CardAdapter> {
    releases.iter().cloned()
        .map(|release| {
            let card: CardAdapter = release_card(&release);
            card
        })
        .collect()
}

fn group_releases_by_release_group(releases: &[Release]) -> Vec<Release> {
    let mut grouped: HashMap<String, Vec<Release>> = HashMap::new();
    let mut ungrouped = Vec::new();
    
    for release in releases {
        if let Some(ref release_group_id) = release.release_group_musicbrainz_id {
            grouped.entry(release_group_id.clone())
                .or_insert_with(Vec::new)
                .push(release.clone());
        } else {
            ungrouped.push(release.clone());
        }
    }
    
    let mut result = Vec::new();
    
    for (_, mut group) in grouped {
        let representative = select_representative_release(&mut group);
        result.push(representative);
    }
    
    result.extend(ungrouped);
    
    result.sort_by(|a, b| {
        match (&b.date, &a.date) {
            (Some(d1), Some(d2)) => d1.cmp(d2),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => match (&a.title, &b.title) {
                (Some(t1), Some(t2)) => t1.cmp(t2),
                _ => std::cmp::Ordering::Equal,
            }
        }
    });
    
    result
}

fn select_representative_release(releases: &mut Vec<Release>) -> Release {
    releases.sort_by(|a, b| {
        let a_score = release_priority_score(a);
        let b_score = release_priority_score(b);
        
        b_score.cmp(&a_score).then_with(|| {
            match (&b.date, &a.date) {
                (Some(d1), Some(d2)) => d1.cmp(d2),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        })
    });
    
    releases.first().unwrap().clone()
}

fn release_priority_score(release: &Release) -> u32 {
    let mut score = 0;
    
    if let Some(ref status) = release.status {
        if status.to_lowercase().contains("official") {
            score += 100;
        }
    }
    
    if let Some(ref country) = release.country {
        match country.as_str() {
            "XW" | "[Worldwide]" => score += 50,
            "US" | "GB" | "EU" => score += 30,
            _ => score += 10,
        }
    }
    
    if release.date.is_some() {
        score += 5;
    }
    
    score
}

fn release_card(release: &Release) -> CardAdapter {
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
            name: format!("{} {}", 
                release.date.unwrap_or_default(), 
                release.release_group_type.unwrap_or_default()).into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}
