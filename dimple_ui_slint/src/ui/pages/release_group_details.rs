use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::Page;
use crate::ui::ReleaseGroupDetailsAdapter;
use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::DimpleEntity;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
use slint::Model as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use url::Url;
use crate::ui::LinkAdapter;
use dimple_db::db::query::QuerySubscription;
use anyhow::Result;

pub struct ReleaseGroupDetailsController {
    release_group_id: MutableStringParam,
    release_id: MutableStringParam,

    release_group_subscription: QuerySubscription,
    artists_subscription: QuerySubscription,
    genres_subscription: QuerySubscription,
    links_subscription: QuerySubscription,
    tracks_subscription: QuerySubscription,
    releases_subscription: QuerySubscription,
}

impl ReleaseGroupDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let release_group_id = MutableStringParam::new();
        let release_id = MutableStringParam::new();
        
        // Set up UI event handlers
        let app_clone = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_play_next(move |key| play_next(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_play_later(move |key| play_later(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_play_track_now(move |key| play_track_now(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_play_track_next(move |key| play_track_next(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_play_track_later(move |key| play_track_later(&app, &key));
            // let ui_weak = ui.as_weak();
            // ui.global::<ReleaseGroupDetailsAdapter>().on_select_version(move |version_index| {
            //     select_version_by_index(&ui_weak, version_index);
            // });
        }).unwrap();
        
        let sql = "SELECT * FROM ReleaseGroup WHERE id = ?";
        let ui = app.ui.clone();
        // let library = app.library.clone();
        let release_group_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |groups: Vec<ReleaseGroup>| {
            if let Some(group) = groups.first() {
                let group = group.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = group.clone().into();
                    ui.global::<ReleaseGroupDetailsAdapter>().set_card(card);
                    ui.global::<ReleaseGroupDetailsAdapter>().set_key(group.id.clone().unwrap_or_default().into());
                    // ui.global::<ReleaseDetailsAdapter>().set_release_type(release.release_group_type.clone().unwrap_or("Release".to_string()).into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_summary(group.summary.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_disambiguation(group.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_dump(serde_json::to_string_pretty(&group).unwrap().into());
                }).unwrap();
            }
        })?;

        // Set up artists subscription  
        let sql = "
            SELECT a.* FROM Artist a
            JOIN ArtistRef ar ON a.id = ar.artist_id
            WHERE ar.model_id = ?
        ";
        let ui = app.ui.clone();
        let artists_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |artists: Vec<Artist>| {
            ui.upgrade_in_event_loop(move |ui| {
                let artist_links = artist_links(&artists);
                ui.global::<ReleaseGroupDetailsAdapter>().set_artists(ModelRc::from(artist_links.as_slice()));
            }).unwrap();
        })?;

        // Set up genres subscription
        let sql = "
            SELECT g.* FROM Genre g
            JOIN GenreRef gr ON g.id = gr.genre_id
            WHERE gr.model_id = ?
        ";
        let ui = app.ui.clone();
        let genres_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |genres: Vec<Genre>| {
            ui.upgrade_in_event_loop(move |ui| {
                let genre_links = genre_links(&genres);
                ui.global::<ReleaseGroupDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
            }).unwrap();
        })?;

        // Set up links subscription
        let sql = "
            SELECT l.* FROM Link l
            JOIN LinkRef lr ON l.id = lr.link_id
            WHERE lr.model_id = ?
        ";
        let ui = app.ui.clone();
        let links_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |links: Vec<Link>| {
            ui.upgrade_in_event_loop(move |ui| {
                let link_adapters = link_links(&links);
                ui.global::<ReleaseGroupDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        let sql = "
            SELECT Release.* 
            FROM Release 
            JOIN ReleaseGroup ON ReleaseGroup.id = Release.release_group_id
            WHERE ReleaseGroup.id = ?
            ORDER BY Release.date ASC, Release.title ASC, Release.rowid ASC
        ";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let releases_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |releases: Vec<Release>| {
            let library = library.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ui.global::<ReleaseGroupDetailsAdapter>();
                let cards = release_cards(&releases, &library);
                adapter.set_releases(ModelRc::from(cards.as_slice()));
            }).unwrap();
        })?;

        // Set up tracks subscription
        let sql = "SELECT * FROM Track WHERE release_id = ? ORDER BY position ASC";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let tracks_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |tracks: Vec<Track>| {
            let library = library.clone();
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<ReleaseGroupDetailsAdapter>().set_track_items(track_items(&library, &tracks));
                ui.global::<ReleaseGroupDetailsAdapter>().set_track_keys(track_keys(&tracks));
            }).unwrap();
        })?;

        Ok(Self {
            release_group_id,
            release_id,
            release_group_subscription,
            artists_subscription,
            genres_subscription,
            links_subscription,
            tracks_subscription,
            releases_subscription,
        })
    }

    pub fn set_release_group(&mut self, release_group_id: String, app: &App) -> Result<()> {
        self.release_group_id.set(&release_group_id);
        
        // Refresh all subscriptions
        // TODO might be worth looking at rxrust for this type of thing
        self.release_group_subscription.refresh();
        self.artists_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.tracks_subscription.refresh();
        self.releases_subscription.refresh();

        // Trigger metadata refresh in background
        let app_clone = app.clone();
        let release_group_id_clone = release_group_id.clone();
        std::thread::spawn(move || {
            if let Some(release_group) = ReleaseGroup::get(&app_clone.library, &release_group_id_clone) {
                librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &release_group.into());
            }
        });

        Ok(())
    }

    pub fn set_release(&mut self, release_id: String, app: &App) -> Result<()> {
        self.release_id.set(&release_id);
        
        // Refresh all subscriptions
        self.release_group_subscription.refresh();
        self.artists_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.tracks_subscription.refresh();
        self.releases_subscription.refresh();

        // Trigger metadata refresh in background
        let app_clone = app.clone();
        let release_id_clone = release_id.clone();
        std::thread::spawn(move || {
            if let Some(release_group) = ReleaseGroup::get(&app_clone.library, &release_id_clone) {
                librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &release_group.into());
            }
        });

        Ok(())
    }
}

pub fn release_group_details(url: &str, app: &App, controller: &mut ReleaseGroupDetailsController) {
    let url = Url::parse(url).unwrap();
    let key = url.path_segments().unwrap().next().unwrap().to_string();

    // Set the release in the controller which will handle all subscriptions
    controller.set_release_group(key, app).unwrap();
    
    // Navigate to the release details page
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::ReleaseGroupDetails);
    }).unwrap();
}

fn play_now(app: &App, key: &str) {
    app.player.play_now(&DimpleEntity::from(&Release::get(&app.library, key).unwrap()));
}

fn play_next(app: &App, key: &str) {
    app.player.play_next(&DimpleEntity::from(&Release::get(&app.library, key).unwrap()));
}

fn play_later(app: &App, key: &str) {
    app.player.play_later(&DimpleEntity::from(&Release::get(&app.library, key).unwrap()));
}

fn play_track_now(app: &App, key: &str) {
    app.player.play_now(&DimpleEntity::from(&Track::get(&app.library, key).unwrap()));
}

fn play_track_next(app: &App, key: &str) {
    app.player.play_next(&DimpleEntity::from(&Track::get(&app.library, key).unwrap()));
}

fn play_track_later(app: &App, key: &str) {
    app.player.play_later(&DimpleEntity::from(&Track::get(&app.library, key).unwrap()));
}

fn track_items(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let track_items: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms))
            .map(format_length);
        row.push(track.position.unwrap_or_default().to_string().as_str().into()); // Track #
        row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
        row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
        row.push(length.unwrap_or_default().as_str().into()); // Length
        track_items.push(row.into());
    }
    track_items.into()
}

fn track_keys(tracks: &[Track]) -> ModelRc<SharedString> {
    let keys: Vec<_> = tracks.iter()
        .map(|track| track.id.clone().unwrap())
        .map(SharedString::from)
        .collect();
    keys.as_slice().into()
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn artist_links(artists: &[Artist]) -> Vec<LinkAdapter> {
    artists.iter().map(|artist| {
        LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn link_links(links: &[Link]) -> Vec<LinkAdapter> {
    links.iter().map(|link| {
        LinkAdapter {
            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
            url: link.url.clone().into(),
        }
    }).collect()
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{minutes}:{seconds:02}")
}

// fn create_version_options(releases: &[Release]) -> Vec<SharedString> {
//     releases.iter().map(|release| {
//         let title = release.title.clone().unwrap_or("Unknown".to_string());
//         let country = release.country.clone().unwrap_or("Unknown".to_string());
//         let date = release.date.clone().unwrap_or("Unknown".to_string());
//         let packaging = release.packaging.clone().unwrap_or("Unknown".to_string());
//         let status = release.status.clone().unwrap_or("Unknown".to_string());
        
//         format!("{} ({}, {}, {}, {})", title, country, date, status, packaging).into()
//     }).collect()
// }

// fn create_version_ids(releases: &[Release]) -> Vec<SharedString> {
//     releases.iter().map(|release| {
//         release.id.clone().unwrap_or_default().into()
//     }).collect()
// }

// fn find_current_version_index(releases: &[Release], current_key: &str) -> i32 {
//     releases.iter()
//         .position(|r| r.id.as_ref().map_or(false, |id| id == current_key))
//         .map(|i| i as i32)
//         .unwrap_or(0)
// }

// fn select_version_by_index(ui_weak: &slint::Weak<crate::ui::AppWindow>, version_index: i32) {
//     if let Some(ui) = ui_weak.upgrade() {
//         let version_ids = ui.global::<ReleaseGroupDetailsAdapter>().get_version_ids();
//         if let Some(release_id) = version_ids.row_data(version_index as usize) {
//             let release_id = release_id.to_string();
//             // Navigate to the new release using the navigator global
//             ui.global::<crate::ui::Navigator>().invoke_navigate(format!("dimple://release/{}", release_id).into());
//         }
//     }
// }

fn release_cards(releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned()
        .map(|release| release_card(&release, &release.artist(library).unwrap_or_default()))
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

