
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use dimple_core::librarian;
use dimple_core::librarian::SearchResults;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Release;
use dimple_core::model::Track;
use dimple_core::plugins;
use dimple_core::plugins::plugins::Plugins;
use dimple_db::db::query::QuerySubscription;
use dimple_db::rusqlite::types::ToSqlOutput;
use dimple_db::rusqlite::ToSql;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardSectionAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::Page;
use crate::ui::SearchResultsAdapter;
use slint::ComponentHandle as _;

pub struct SearchResultsController {
    _sub: QuerySubscription,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SearchResult {
    rank: f32,
    id: String,
    entity_type: String,
    title: String,
    sub_title: String,
}

impl SearchResultsController {
    pub fn new(app: &App) -> Result<Self> {
        let ui = app.ui.clone();
        let sql = "
            SELECT 0 AS rank, id AS id, 'Artist' AS entity_type, name AS title, coalesce(disambiguation, '') AS sub_title FROM Artist WHERE name LIKE ?1
            UNION 
            SELECT 0 AS rank, id AS id, 'Release' AS entity_type, title AS title, coalesce(release_group_type, '') AS sub_title FROM Release WHERE title LIKE ?1
            UNION 
            SELECT 0 AS rank, id AS id, 'Genre' AS entity_type, name AS title, coalesce(disambiguation, '') AS sub_title FROM Genre WHERE name LIKE ?1
            UNION 
            SELECT 0 AS rank, id AS id, 'Track' AS entity_type, title AS title, coalesce(disambiguation, '') AS sub_title FROM Track WHERE title LIKE ?1
            ORDER BY rank, title
            LIMIT 25
        ";
        let query_param = MutableStringParam::new();
        let app_clone = app.clone();
        let sub = app.library.db.query_subscribe(sql, (query_param.clone(),), move |results: Vec<SearchResult>| {
            log::info!("Results refreshed: {} results", results.len());
            let results = results.into_iter().into_group_map_by(|f| f.entity_type.clone());
            // TODO this conversion back to entities is legacy and not needed now. 
            // Can just create a SearchResult card. Which will then line up nicely
            // with FTS. Still want to break them up into sections though - I like
            // that.
            let results = SearchResults {
                artists: results.get("Artist").cloned().unwrap_or_default().iter().map(|f| Artist {
                    id: Some(f.id.clone()),
                    name: Some(f.title.clone()),
                    disambiguation: Some(f.sub_title.clone()),
                    ..Default::default()
                }).collect(),
                releases: results.get("Release").cloned().unwrap_or_default().iter().map(|f| Release {
                    id: Some(f.id.clone()),
                    title: Some(f.title.clone()),
                    disambiguation: Some(f.sub_title.clone()),
                    ..Default::default()
                }).collect(),
                tracks: results.get("Track").cloned().unwrap_or_default().iter().map(|f| Track {
                    id: Some(f.id.clone()),
                    title: Some(f.title.clone()),
                    disambiguation: Some(f.sub_title.clone()),
                    ..Default::default()
                }).collect(),
                genres: results.get("Genre").cloned().unwrap_or_default().iter().map(|f| Genre {
                    id: Some(f.id.clone()),
                    name: Some(f.title.clone()),
                    disambiguation: Some(f.sub_title.clone()),
                    ..Default::default()
                }).collect(),
                ..Default::default()
            };
            update_results(&app_clone, results);
        })?;

        let ui_clone = ui.clone();
        let sub_clone = sub.clone();
        let plugins_clone = app.plugins.clone();
        let library_clone = app.library.clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<SearchResultsAdapter>().on_query(move |query| {
                let query_str = format!("%{}%", query);
                query_param.set(&query_str);
                sub_clone.refresh();            
                search_plugins(plugins_clone.clone(), library_clone.clone(), query_str);
                ui_clone.upgrade_in_event_loop(move |ui| ui.set_page(Page::SearchResults)).unwrap();
            });
                
        }).unwrap();
        
        Ok(Self {
            _sub: sub,
        })
    }
}

fn search_plugins(plugins: Plugins, library: Library, query: String) {
    thread::spawn(move || {
        let plugin_results = plugins.search(&library, &query);

        for result in plugin_results {
            for artist in result.artists {
                librarian::merge_artist(&library, &artist);
            }
            for release in result.releases {
                // TODO
                // librarian::merge_release_metadata(&library, &release, None);
            }
        }
    });
}

fn update_results(app: &App, results: SearchResults) {
    let artists = results.artists;
    let tracks = results.tracks;
    let genres = results.genres;
    let releases = results.releases;
                                
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let mut sections: Vec<CardSectionAdapter> = vec![];

        if !tracks.is_empty() {
            sections.push(CardSectionAdapter {
                title: "Tracks".into(),
                sub_title: Default::default(),
                cards: track_cards(&app.images, &tracks, &app.library).as_slice().into(),
                ..Default::default()
            });
        }

        if !artists.is_empty() {
            sections.push(CardSectionAdapter {
                title: "Artists".into(),
                sub_title: Default::default(),
                cards: artist_cards(&app.images, &artists).as_slice().into(),
                ..Default::default()
            });
        }

        if !releases.is_empty() {
            sections.push(CardSectionAdapter {
                title: "Releases".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &releases, &app.library).as_slice().into(),
                ..Default::default()
            });
        }

        if !genres.is_empty() {
            sections.push(CardSectionAdapter {
                title: "Genres".into(),
                sub_title: Default::default(),
                cards: genre_cards(&app.images, &genres).as_slice().into(),
                ..Default::default()
            });
        }

        let adapter = ui.global::<SearchResultsAdapter>();
        adapter.set_sections(sections.as_slice().into());
    }).unwrap();
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(&release.into(), 200, 200, move |ui, image| {
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

fn artist_cards(images: &ImageMangler, artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let mut card: CardAdapter = artist_card(&artist);
            card.image.image = images.lazy_get(&artist.into(), 200, 200, move |ui, image| {
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

fn genre_cards(images: &ImageMangler, genres: &[Genre]) -> Vec<CardAdapter> {
    genres.iter().cloned().enumerate()
        .map(|(index, genre)| {
            let mut card: CardAdapter = genre_card(&genre);
            card.image.image = images.lazy_get(&genre.into(), 200, 200, move |ui, image| {
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

fn track_cards(images: &ImageMangler, tracks: &[Track], library: &Library) -> Vec<CardAdapter> {
    tracks.iter().cloned().enumerate()
        .map(|(index, track)| {
            let mut card: CardAdapter = track_card(&track, &track.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(&track.into(), 200, 200, move |ui, image| {
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
        key: track.id.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

#[derive(Clone)]
struct MutableStringParam {
    value: Arc<Mutex<String>>,
}

impl MutableStringParam {
    pub fn new() -> Self {
        MutableStringParam { 
            value: Arc::new(Mutex::new("".to_string())) 
        }
    }

    pub fn set(&self, value: &str) {
        *self.value.lock().unwrap() = value.to_string();
    }
}

impl ToSql for MutableStringParam {
    fn to_sql(&self) -> dimple_db::rusqlite::Result<ToSqlOutput<'_>> {
        let s = self.value.lock().unwrap().to_string();
        let t = ToSqlOutput::Owned(dimple_db::rusqlite::types::Value::Text(s));
        Ok(t)
    }
}

