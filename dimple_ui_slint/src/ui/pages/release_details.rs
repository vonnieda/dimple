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
        }).unwrap();
        
        // Set up release subscription
        let sql = "SELECT * FROM Release WHERE id = ?";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let release_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |releases: Vec<Release>| {
            if let Some(release) = releases.first() {
                let release = release.clone();
                let library = library.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = release.clone().into();

                    ui.global::<ReleaseDetailsAdapter>().set_card(card.into());
                    ui.global::<ReleaseDetailsAdapter>().set_key(release.id.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_release_type(release.release_group_type.clone().unwrap_or("Release".to_string()).into());
                    ui.global::<ReleaseDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_dump(format!("{:?}", release).into());
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

        Ok(Self {
            current_key,
            release_subscription,
            artists_subscription,
            genres_subscription,
            links_subscription,
            tracks_subscription,
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
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();

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
            .map(|ms| Duration::from_millis(ms as u64))
            .map(|dur| format_length(dur));
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
        .map(|key| SharedString::from(key))
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
    format!("{}:{:02}", minutes, seconds)
}

