use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::DimpleEntity;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics as _;
use dimple_core::model::Release;
use dimple_core::model::Track;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::ImageLinkAdapter;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
use crate::ui::LinkAdapter;
use slint::ComponentHandle as _;
use crate::ui::CardAdapter;
use dimple_db::db::query::QuerySubscription;
use anyhow::Result;

pub struct NowPlayingController {
    current_key: MutableStringParam,
    track_subscription: QuerySubscription,
    artists_subscription: QuerySubscription,
    genres_subscription: QuerySubscription,
    links_subscription: QuerySubscription,
    release_subscription: QuerySubscription,
}

impl NowPlayingController {
    pub fn new(app: &App) -> Result<Self> {
        let current_key = MutableStringParam::new();
        
        // Set up UI event handlers
        let app_clone = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_clone.clone();
            ui.global::<TrackDetailsAdapter>().on_set_lyrics(move |key, lyrics| set_lyrics(&app, &key, &lyrics));
        }).unwrap();
        
        // Set up track subscription
        let sql = "SELECT * FROM Track WHERE id = ?";
        let ui = app.ui.clone();
        let track_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |tracks: Vec<Track>| {
            if let Some(track) = tracks.first() {
                let track = track.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = track.clone().into();
                    ui.global::<TrackDetailsAdapter>().set_card(card);
                    ui.global::<TrackDetailsAdapter>().set_key(track.id.clone().unwrap_or_default().into());
                    ui.global::<TrackDetailsAdapter>().set_summary(track.summary.clone().unwrap_or_default().into());
                    ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());
                    let lyrics = track.lyrics.clone()
                        .map(|s| s.trim().replace("\r", ""))
                        .filter(|s| !s.is_empty())
                        .unwrap_or("(No lyrics, click title to edit.)".to_string());
                    ui.global::<TrackDetailsAdapter>().set_lyrics(lyrics.into());
                    ui.global::<TrackDetailsAdapter>().set_dump(serde_json::to_string_pretty(&track).unwrap().into());
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
                ui.global::<TrackDetailsAdapter>().set_artists(ModelRc::from(artist_links.as_slice()));
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
                ui.global::<TrackDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
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
                ui.global::<TrackDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        // Set up release subscription
        let sql = "SELECT * FROM Release WHERE id = (SELECT release_id FROM Track WHERE id = ?)";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let release_subscription = app.library.db.query_subscribe(sql, (current_key.clone(),), move |releases: Vec<Release>| {
            let library = library.clone();
            ui.upgrade_in_event_loop(move |ui| {
                if let Some(release) = releases.first() {
                    ui.global::<TrackDetailsAdapter>().set_release_date(release.date.clone().unwrap_or_default().into());
                    let release_cards = release_cards(&[release.clone()], &library);
                    ui.global::<TrackDetailsAdapter>().set_releases(release_cards.as_slice().into());
                }
            }).unwrap();
        })?;

        Ok(Self {
            current_key,
            track_subscription,
            artists_subscription,
            genres_subscription,
            links_subscription,
            release_subscription,
        })
    }

    pub fn set_track(&mut self, key: String, app: &App) -> Result<()> {
        self.current_key.set(&key);
        
        // Refresh all subscriptions
        self.track_subscription.refresh();
        self.artists_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.release_subscription.refresh();

        // Trigger metadata refresh in background
        let app_clone = app.clone();
        let key_clone = key.clone();
        std::thread::spawn(move || {
            if let Some(track) = Track::get(&app_clone.library, &key_clone) {
                librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &DimpleEntity::from(&track));
            }
        });

        Ok(())
    }
}

// TODO I suppose this is really just self.show()
// or navigate and we also have show()
pub fn now_playing(url: &str, app: &App, controller: &mut NowPlayingController) {
    let url = Url::parse(url).unwrap();
    let key = url.path_segments().unwrap().next().unwrap().to_string();

    // Set the track in the controller which will handle all subscriptions
    controller.set_track(key, app).unwrap();
    
    // Navigate to the track details page
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::TrackDetails);
    }).unwrap();
}

fn set_lyrics(app: &App, key: &str, lyrics: &str) {
    let mut track = app.library.get::<Track>(key).unwrap();
    track.lyrics = Some(lyrics.to_string());
    let _ = app.library.save(&track);
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

fn release_cards(releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
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
            name: release.date.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}
