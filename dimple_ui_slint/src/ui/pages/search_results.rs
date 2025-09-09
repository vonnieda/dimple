
use std::collections::HashMap;
use std::thread;

use anyhow::Result;
use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::plugins::plugins::Plugins;
use dimple_db::db::query::QuerySubscription;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
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
        // TODO want to use MATCH instead of = (does it matter?) but MATCH
        // blows up the query parser.
        let sql = "
            SELECT 
                bm25(ArtistFts) AS rank, 
                Artist.id AS id, 
                'Artist' AS entity_type, 
                Artist.name AS title, 
                COALESCE(Artist.disambiguation, '') AS sub_title
            FROM ArtistFts 
            JOIN Artist ON Artist.rowid = ArtistFts.rowid 
            WHERE ArtistFts = ?1

            UNION

            SELECT 
                bm25(ReleaseFts) AS rank, 
                Release.id AS id, 
                'Release' AS entity_type, 
                Release.title AS title, 
                '' AS sub_title 
            FROM ReleaseFts 
            JOIN Release ON Release.rowid = ReleaseFts.rowid 
            WHERE ReleaseFts = ?1

            UNION

            SELECT 
                bm25(ReleaseGroupFts) AS rank, 
                ReleaseGroup.id AS id, 
                'ReleaseGroup' AS entity_type, 
                ReleaseGroup.title AS title, 
                COALESCE(ReleaseGroup.primary_type, '') AS sub_title 
            FROM ReleaseGroupFts 
            JOIN ReleaseGroup ON ReleaseGroup.rowid = ReleaseGroupFts.rowid 
            WHERE ReleaseGroupFts = ?1

            UNION

            SELECT 
                bm25(TrackFts) AS rank, 
                Track.id AS id, 
                'Track' AS entity_type, 
                Track.title AS title, 
                COALESCE(Track.disambiguation, '') AS sub_title 
            FROM TrackFts 
            JOIN Track ON Track.rowid = TrackFts.rowid 
            WHERE TrackFts = ?1

            UNION

            SELECT 
                bm25(GenreFts) AS rank, 
                Genre.id AS id, 
                'Genre' AS entity_type, 
                Genre.name AS title, 
                COALESCE(Genre.disambiguation, '') AS sub_title 
            FROM GenreFts 
            JOIN Genre ON Genre.rowid = GenreFts.rowid 
            WHERE GenreFts = ?1

            ORDER BY rank, title
        ";

        let query_param = MutableStringParam::new();
        // TODO blows up the query if empty, can probably fix in query with coalesce
        query_param.set("c3bb4692169bx7361bc8914b6c9b1239c4");
        let app_clone = app.clone();
        let sub = app.library.db.query_subscribe(sql, (query_param.clone(),), move |results: Vec<SearchResult>| {
            let results = results.into_iter().into_group_map_by(|f| f.entity_type.clone());
            update_results(&app_clone, results);
        })?;

        let ui_clone = ui.clone();
        let sub_clone = sub.clone();
        let plugins_clone = app.plugins.clone();
        let library_clone = app.library.clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<SearchResultsAdapter>().on_query(move |query| {
                query_param.set(&query);
                sub_clone.refresh();            
                search_plugins(plugins_clone.clone(), library_clone.clone(), query.to_string());
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

        library.db.transaction(|txn| {
            for result in plugin_results {
                for artist in result.artists {
                    librarian::merge_artist_metadata(txn, &artist, None)?;
                }
                for release in result.releases {
                    librarian::merge_release_metadata(txn, &release, None)?;
                }
                for release_group in result.release_groups {
                    librarian::merge_release_group_metadata(txn, &release_group, None)?;
                }
                for track in result.tracks {
                    librarian::merge_track_metadata(txn, &track, None)?;
                }
                for genre in result.genres {
                    librarian::merge_genre(txn, &genre)?;
                }
            }
            Ok(())
        }).unwrap_or_else(|e| {
            eprintln!("Failed to merge search results: {e}");
        });
    });
}

fn update_results(app: &App, results: HashMap<String, Vec<SearchResult>>) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let mut sections: Vec<CardSectionAdapter> = vec![];

        // TODO DRY
        if let Some(artist_results) = results.get("Artist") {
            if !artist_results.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Artists".into(),
                    sub_title: Default::default(),
                    cards: search_result_cards(artist_results).as_slice().into(),
                    ..Default::default()
                });
            }
        }
        if let Some(release_results) = results.get("Release") {
            if !release_results.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Releases".into(),
                    sub_title: Default::default(),
                    cards: search_result_cards(release_results).as_slice().into(),
                    ..Default::default()
                });
            }
        }
        if let Some(release_groups_results) = results.get("ReleaseGroup") {
            if !release_groups_results.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Release Groups".into(),
                    sub_title: Default::default(),
                    cards: search_result_cards(release_groups_results).as_slice().into(),
                    ..Default::default()
                });
            }
        }
        if let Some(track_results) = results.get("Track") {
            if !track_results.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Tracks".into(),
                    sub_title: Default::default(),
                    cards: search_result_cards(track_results).as_slice().into(),
                    ..Default::default()
                });
            }
        }
        if let Some(genre_results) = results.get("Genre") {
            if !genre_results.is_empty() {
                sections.push(CardSectionAdapter {
                    title: "Genres".into(),
                    sub_title: Default::default(),
                    cards: search_result_cards(genre_results).as_slice().into(),
                    ..Default::default()
                });
            }
        }

        let adapter = ui.global::<SearchResultsAdapter>();
        adapter.set_sections(sections.as_slice().into());
    }).unwrap();
}

fn search_result_cards(results: &Vec<SearchResult>) -> Vec<CardAdapter> {
    results.iter().map(search_result_card).collect()
}

fn search_result_card(result: &SearchResult) -> CardAdapter {
    let url = &format!("dimple://{}/{}", result.entity_type.to_lowercase(), &result.id);
    let title = &result.title;
    let sub_title = &result.sub_title;
    let key = &result.id;
    CardAdapter {
        key: key.into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: title.into(),
            url: url.into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: title.into(),
            url: url.into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: sub_title.into(),
            url: url.into(),
        },
        ..Default::default()
    }
}

