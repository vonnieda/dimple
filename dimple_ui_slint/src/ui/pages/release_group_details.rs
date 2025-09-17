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
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use slint::ComponentHandle as _;
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
        let release_group_id_clone = release_group_id.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_clone.clone();
            ui.global::<ReleaseGroupDetailsAdapter>().on_toggle_heart(move || {
                app.library.db.transaction(|txn| {
                    let mut release_group: ReleaseGroup = txn.get(&release_group_id_clone.value())?.expect("release group not found");
                    release_group.save = !release_group.save;
                    Ok(txn.save(&release_group)?)
                }).unwrap();
            });
        })?;
        
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
                    ui.global::<ReleaseGroupDetailsAdapter>().set_save(group.save);
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
            SELECT Link.* FROM Link
            JOIN LinkRef ON Link.id = LinkRef.link_id
            WHERE LinkRef.model_id = ?
            ORDER BY Link.name, Link.url, Link.id
        ";
        let ui = app.ui.clone();
        let links_subscription = app.library.db.query_subscribe(sql, (release_group_id.clone(),), move |links: Vec<Link>| {
            ui.upgrade_in_event_loop(move |ui| {
                let link_adapters = link_links(&links);
                ui.global::<ReleaseGroupDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        // Set up tracks subscription
        let sql = "
            SELECT * 
            FROM Track 
            WHERE release_id = ? 
            ORDER BY position ASC
        ";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let tracks_subscription = app.library.db.query_subscribe(sql, (release_id.clone(),), move |tracks: Vec<Track>| {
            let library = library.clone();
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<ReleaseGroupDetailsAdapter>().set_track_items(track_items(&library, &tracks));
                ui.global::<ReleaseGroupDetailsAdapter>().set_track_keys(track_keys(&tracks));
            }).unwrap();
        })?;

        // Set up releases (versions) subscription
        let sql = "
            SELECT Release.* 
            FROM Release 
            WHERE Release.release_group_id = ?
            ORDER BY Release.date ASC NULLS LAST, Release.title ASC, Release.id ASC
        ";
        let ui = app.ui.clone();
        let release_id_clone = release_id.clone();
        let tracks_subscription_clone = tracks_subscription.clone();
        let releases_subscription = app.library.db.query_subscribe(sql, 
            (release_group_id.clone(),), move |releases: Vec<Release>| {

            if let Some(release) = releases.get(0) {
                release_id_clone.set(release.id.clone().unwrap().as_ref());
            }
            else {
                release_id_clone.set("");
            }
            tracks_subscription_clone.refresh();

            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ui.global::<ReleaseGroupDetailsAdapter>();
                let cards = release_version_cards(&releases);
                adapter.set_releases(ModelRc::from(cards.as_slice()));
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

    pub fn set_release_group_id(&self, release_group_id: &str, app: &App) {
        let release_group = ReleaseGroup::get(&app.library, &release_group_id).unwrap();
        self.release_group_id.set(&release_group.id.clone().unwrap());
        // Changing the release_group_id means we need to reload the releases,
        // and pick one, so clear that and it will happen in the subcription
        // callbacks.
        self.release_id.set("");

        self.release_group_subscription.refresh();
        self.artists_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.tracks_subscription.refresh();
        self.releases_subscription.refresh();

        let app_clone = app.clone();
        std::thread::spawn(move || {
            librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &release_group.into());
            // if let Some(release) = releases.get(0) {
            //     // TODO should be on selected release
            //     librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &release.into());
            // }
        });
    }

    pub fn navigate(&self, url: &str, app: &App) {
        let url = Url::parse(url).unwrap();
        let release_group_id = url.path_segments().unwrap().next().unwrap().to_string();
        self.set_release_group_id(&release_group_id, app);

        app.ui.upgrade_in_event_loop(move |ui| {
            ui.set_page(Page::ReleaseGroupDetails);
        }).unwrap();
    }
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

fn release_version_cards(releases: &[Release]) -> Vec<CardAdapter> {
    releases.iter().cloned()
        .map(|release| release_version_card(&release))
        .collect()
}

fn release_version_card(release: &Release) -> CardAdapter {
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
            name: format!("{} • {}", 
                release.date.clone().unwrap_or_default(), 
                release.country.clone().unwrap_or_default()).into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: release.disambiguation.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

