use std::sync::Mutex;

use dimple_core::library::Library;
use dimple_core::merge::CrdtRules;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::Release;
use dimple_core::model::Track;
use dimple_core::plugins::plugin_host::PluginHost;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
use crate::ui::LinkAdapter;
use slint::ComponentHandle as _;
use crate::ui::CardAdapter;

// TODO how do I make the poop emoji?
static CURRENT_TRACK_KEY: Mutex<Option<String>> = Mutex::new(None);

pub fn track_details_init(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app1.clone();
        ui.global::<TrackDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = app1.clone();
        ui.global::<TrackDetailsAdapter>().on_add_to_queue(move |key| add_to_queue(&app, &key));
        let app = app1.clone();
        ui.global::<TrackDetailsAdapter>().on_set_lyrics(move |key, lyrics| set_lyrics(&app, &key, &lyrics));
    }).unwrap();

    let app1 = app.clone();
    app.library.on_change(Box::new(move |event| {
        if let Some(current_track_key) = CURRENT_TRACK_KEY.lock().unwrap().clone() {
            if current_track_key == event.key {
                let track = app1.library.get::<Track>(&event.key).unwrap();
                app1.ui.upgrade_in_event_loop(move |ui| {
                    let lyrics = track.lyrics.clone()
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or("(No lyrics, click to edit.)".to_string());
                    ui.global::<TrackDetailsAdapter>().set_lyrics(lyrics.into());
                }).unwrap();
            }
        }
    }));
}

pub fn track_details(url: &str, app: &App) {
    let url = url.to_owned();
    let app = app.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();
        let library = app.library.clone();

        let track = app.library.get::<Track>(key).unwrap();
        (*CURRENT_TRACK_KEY.lock().unwrap()) = track.key.clone();

        {
            // TODO temp for testing, not sure where this is gonna live yet.
            let track = track.clone();
            let library = library.clone();
            std::thread::spawn(move || {
                refresh_lyrics(&library, &app.plugins, &track);
            });
        }

        let artists = track.artists(&library);
        let genres: Vec<Genre> = track.genres(&library);
        let links: Vec<Link> = track.links(&library);
        let release = track.release(&library).unwrap_or_default();
        app.ui.upgrade_in_event_loop(move |ui| {
            let artists = artist_links(&artists);
            let genres = genre_links(&genres);
            let links = link_links(&links);

            let mut card: CardAdapter = track.clone().into();
            card.image.image = app.images.lazy_get(track.clone(), 275, 275, |ui, image| {
                let mut card = ui.global::<TrackDetailsAdapter>().get_card();
                card.image.image = image;
                ui.global::<TrackDetailsAdapter>().set_card(card);
            });

            ui.global::<TrackDetailsAdapter>().set_card(card.into());
            ui.global::<TrackDetailsAdapter>().set_key(track.key.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_artists(ModelRc::from(artists.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_summary(track.summary.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_release(release.clone().into());
            ui.global::<TrackDetailsAdapter>().set_release_date(release.date.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());            
            ui.global::<TrackDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
            let lyrics = track.lyrics.clone()
                .map(|s| s.trim().replace("\r", ""))
                .filter(|s| !s.is_empty())
                .unwrap_or("(No lyrics, click to edit.)".to_string());
            ui.global::<TrackDetailsAdapter>().set_lyrics(lyrics.into());
            ui.global::<TrackDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_dump(format!("{:?}", track).into());

            ui.set_page(Page::TrackDetails);
        }).unwrap();
    });
}

fn refresh_lyrics(library: &Library, plugins: &PluginHost, track: &Track) {
    let lyrics = plugins.lyrics(library, track)
        .into_iter()
        .reduce(CrdtRules::merge);
    if let Some(mut track) = library.get::<Track>(&track.key.clone().unwrap()) {
        track.lyrics = CrdtRules::merge(track.lyrics, lyrics);
        library.save(&track);
    }
}

fn play_now(app: &App, key: &str) {
    let app = app.clone();
    let key = key.to_string();
    app.ui.upgrade_in_event_loop(move |ui| {
        // TODO think about ephemeral or secondary playlist, or even
        // a playlist inserted inbetween the playing items
        let play_queue = app.player.queue();
        app.library.playlist_add(&play_queue, &key);
        let len = play_queue.len(&app.library);
        app.player.set_queue_index(len - 1);
        app.player.play();
    }).unwrap();
}

fn add_to_queue(app: &App, key: &str) {
    let app = app.clone();
    let key = key.to_string();
    app.ui.upgrade_in_event_loop(move |ui| {
        let play_queue = app.player.queue();
        app.library.playlist_add(&play_queue, &key);
        app.player.play();
    }).unwrap();
}

fn set_lyrics(app: &App, key: &str, lyrics: &str) {
    let mut track = app.library.get::<Track>(key).unwrap();
    track.lyrics = Some(lyrics.to_string());
    app.library.save(&track);
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn artist_links(artists: &[Artist]) -> Vec<LinkAdapter> {
    artists.iter().map(|artist| {
        LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
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

impl From<Release> for LinkAdapter {
    fn from(value: Release) -> Self {
        LinkAdapter {
            name: value.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
        }
    }
}