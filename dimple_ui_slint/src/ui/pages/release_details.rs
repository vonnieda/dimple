use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::DimpleEntity;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
use slint::Model as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseDetailsAdapter;
use dimple_db::db::query::QuerySubscription;
use anyhow::Result;

pub struct ReleaseDetailsController {
    current_key: MutableStringParam,
    release_subscription: QuerySubscription,
    artists_subscription: QuerySubscription,
    genres_subscription: QuerySubscription,
    links_subscription: QuerySubscription,
    tracks_subscription: QuerySubscription,
    versions_subscription: QuerySubscription,
}

impl ReleaseDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let current_key = MutableStringParam::new();
        
        // TODO okay the cause of the not being able to right click a release
        // track and play now immediately after import is cause we're
        // not setting the key in the ui or maybe cause the query is not
        // reloading how it used to.
        // row-menu-key = ReleaseDetailsAdapter.row_keys[row];


        // Set up UI event handlers
        let app_clone = app.clone();
        let current_key_clone = current_key.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_clone.clone();
            ui.global::<ReleaseDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseDetailsAdapter>().on_play_next(move |key| play_next(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseDetailsAdapter>().on_play_later(move |key| play_later(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseDetailsAdapter>().on_play_track_now(move |key| play_track_now(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseDetailsAdapter>().on_play_track_next(move |key| play_track_next(&app, &key));
            let app = app_clone.clone();
            ui.global::<ReleaseDetailsAdapter>().on_play_track_later(move |key| play_track_later(&app, &key));
            let ui_weak = ui.as_weak();
            ui.global::<ReleaseDetailsAdapter>().on_select_version(move |version_index| {
                select_version_by_index(&ui_weak, version_index);
            });
        }).unwrap();
        
        // Set up release subscription
        let sql = "SELECT * FROM Release WHERE id = ?";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let release_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |releases: Vec<Release>| {
            if let Some(release) = releases.first() {
                let release = release.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = release.clone().into();
                    ui.global::<ReleaseDetailsAdapter>().set_card(card);
                    ui.global::<ReleaseDetailsAdapter>().set_key(release.id.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_release_type(release.release_group_type.clone().unwrap_or("Release".to_string()).into());
                    ui.global::<ReleaseDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_dump(serde_json::to_string_pretty(&release).unwrap().into());
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
        let artists_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |artists: Vec<Artist>| {
            ui.upgrade_in_event_loop(move |ui| {
                let artist_links = artist_links(&artists);
                ui.global::<ReleaseDetailsAdapter>().set_artists(ModelRc::from(artist_links.as_slice()));
            }).unwrap();
        })?;

        // Set up genres subscription
        let sql = "
            SELECT g.* FROM Genre g
            JOIN GenreRef gr ON g.id = gr.genre_id
            WHERE gr.model_id = ?
        ";
        let ui = app.ui.clone();
        let genres_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |genres: Vec<Genre>| {
            ui.upgrade_in_event_loop(move |ui| {
                let genre_links = genre_links(&genres);
                ui.global::<ReleaseDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
            }).unwrap();
        })?;

        // Set up links subscription
        let sql = "
            SELECT l.* FROM Link l
            JOIN LinkRef lr ON l.id = lr.link_id
            WHERE lr.model_id = ?
        ";
        let ui = app.ui.clone();
        let links_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |links: Vec<Link>| {
            ui.upgrade_in_event_loop(move |ui| {
                let link_adapters = link_links(&links);
                ui.global::<ReleaseDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        // Set up tracks subscription
        let sql = "SELECT * FROM Track WHERE release_id = ? ORDER BY position ASC";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let tracks_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |tracks: Vec<Track>| {
            let library = library.clone();
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<ReleaseDetailsAdapter>().set_row_data(row_data(&library, &tracks));
                ui.global::<ReleaseDetailsAdapter>().set_row_keys(row_keys(&tracks));
            }).unwrap();
        })?;

        // Set up versions subscription - find all releases in the same release group
        let sql = "
            SELECT r.* FROM Release r 
            JOIN Release current ON current.id = ? 
            WHERE r.release_group_musicbrainz_id = current.release_group_musicbrainz_id 
            AND r.release_group_musicbrainz_id IS NOT NULL
            ORDER BY 
                r.date ASC,
                CASE WHEN r.status LIKE '%official%' THEN 0 ELSE 1 END,
                CASE WHEN r.country IN ('XW', '[Worldwide]') THEN 0 
                     WHEN r.country IN ('US', 'GB', 'EU') THEN 1 ELSE 2 END,
                r.title ASC
        ";
        let ui = app.ui.clone();
        let versions_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |releases: Vec<Release>| {
            ui.upgrade_in_event_loop(move |ui| {
                let version_options = create_version_options(&releases);
                let version_ids = create_version_ids(&releases);
                let current_key = ui.global::<ReleaseDetailsAdapter>().get_key().to_string();
                let current_index = find_current_version_index(&releases, &current_key);
                
                ui.global::<ReleaseDetailsAdapter>().set_version_options(ModelRc::from(version_options.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_version_ids(ModelRc::from(version_ids.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_version_index(current_index);
            }).unwrap();
        })?;

        Ok(Self {
            current_key,
            release_subscription,
            artists_subscription,
            genres_subscription,
            links_subscription,
            tracks_subscription,
            versions_subscription,
        })
    }

    pub fn set_release(&mut self, key: String, app: &App) -> Result<()> {
        self.current_key.set(&key);
        
        // Refresh all subscriptions
        self.release_subscription.refresh();
        self.artists_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.tracks_subscription.refresh();
        self.versions_subscription.refresh();

        // Trigger metadata refresh in background
        let app_clone = app.clone();
        let key_clone = key.clone();
        std::thread::spawn(move || {
            if let Some(release) = Release::get(&app_clone.library, &key_clone) {
                librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &release.into());
            }
        });

        Ok(())
    }
}

pub fn release_details(url: &str, app: &App, controller: &mut ReleaseDetailsController) {
    let url = Url::parse(url).unwrap();
    let key = url.path_segments().unwrap().next().unwrap().to_string();

    // Set the release in the controller which will handle all subscriptions
    controller.set_release(key, app).unwrap();
    
    // Navigate to the release details page
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::ReleaseDetails);
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

fn row_data(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
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
        row_data.push(row.into());
    }
    row_data.into()
}

fn row_keys(tracks: &[Track]) -> ModelRc<SharedString> {
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

fn create_version_options(releases: &[Release]) -> Vec<SharedString> {
    releases.iter().map(|release| {
        let title = release.title.clone().unwrap_or("Unknown".to_string());
        let country = release.country.clone().unwrap_or("Unknown".to_string());
        let date = release.date.clone().unwrap_or("Unknown".to_string());
        let packaging = release.packaging.clone().unwrap_or("Unknown".to_string());
        let status = release.status.clone().unwrap_or("Unknown".to_string());
        
        format!("{} ({}, {}, {}, {})", title, country, date, status, packaging).into()
    }).collect()
}

fn create_version_ids(releases: &[Release]) -> Vec<SharedString> {
    releases.iter().map(|release| {
        release.id.clone().unwrap_or_default().into()
    }).collect()
}

fn find_current_version_index(releases: &[Release], current_key: &str) -> i32 {
    releases.iter()
        .position(|r| r.id.as_ref().map_or(false, |id| id == current_key))
        .map(|i| i as i32)
        .unwrap_or(0)
}

fn select_version_by_index(ui_weak: &slint::Weak<crate::ui::AppWindow>, version_index: i32) {
    if let Some(ui) = ui_weak.upgrade() {
        let version_ids = ui.global::<ReleaseDetailsAdapter>().get_version_ids();
        if let Some(release_id) = version_ids.row_data(version_index as usize) {
            let release_id = release_id.to_string();
            // Navigate to the new release using the navigator global
            ui.global::<crate::ui::Navigator>().invoke_navigate(format!("dimple://release/{}", release_id).into());
        }
    }
}

