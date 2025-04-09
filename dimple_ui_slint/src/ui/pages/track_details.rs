use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics as _;
use dimple_core::model::Release;
use dimple_core::model::Track;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::ImageLinkAdapter;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
use crate::ui::LinkAdapter;
use slint::ComponentHandle as _;
use crate::ui::CardAdapter;

pub fn track_details_init(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app1.clone();
        ui.global::<TrackDetailsAdapter>().on_set_lyrics(move |key, lyrics| set_lyrics(&app, &key, &lyrics));
    }).unwrap();

    // TODO filter events by key - but we can't get the key without the
    // UI, so rethink the whole mess.
    let app1 = app.clone();
    app.library.notifier.observe(move |event| if event.type_name == "Track" { update_model(&app1) });
}

pub fn track_details(url: &str, app: &App) {
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();

    let app1 = app.clone();
    let key1 = key.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<TrackDetailsAdapter>().set_key(key1.clone().into());
        update_model(&app1);
        ui.set_page(Page::TrackDetails);

        let app2 = app1.clone();
        let key2 = key1.clone();
        std::thread::spawn(move || {
            if let Some(track) = Track::get(&app2.library, &key2) {
                librarian::refresh_metadata(&app2.library, &app2.plugins, &track);
            }
        });    
    }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<TrackDetailsAdapter>().get_key();
        if key.is_empty() {
            return
        }
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let track = Track::get(&library, &key).unwrap();
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
                ui.global::<TrackDetailsAdapter>().set_release_date(release.date.clone().unwrap_or_default().into());
                ui.global::<TrackDetailsAdapter>().set_releases(release_cards(&app.images, &[release], &app.library).as_slice().into());
                ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());            
                ui.global::<TrackDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
                let lyrics = track.lyrics.clone()
                    .map(|s| s.trim().replace("\r", ""))
                    .filter(|s| !s.is_empty())
                    .unwrap_or("(No lyrics, click title to edit.)".to_string());
                ui.global::<TrackDetailsAdapter>().set_lyrics(lyrics.into());
                ui.global::<TrackDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<TrackDetailsAdapter>().set_dump(format!("{:?}", track).into());
    
                ui.set_page(Page::TrackDetails);
            }).unwrap();
        });
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

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(release.clone(), 200, 200, move |ui, image| {
                let adapter = ui.global::<TrackDetailsAdapter>();
                let mut card = adapter.get_releases().row_data(index).unwrap();
                card.image.image = image;
                adapter.get_releases().set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: release.date.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}
