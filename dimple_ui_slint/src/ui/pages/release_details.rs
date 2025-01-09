use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use slint::ComponentHandle as _;
use slint::ModelRc;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseDetailsAdapter;

pub fn release_details_init(app: &App) {
    let app = app.clone();
    let library = app.library.clone();
    // TODO filter events
    library.on_change(Box::new(move |_event| update_model(&app)));
}

pub fn release_details(url: &str, app: &App) {
    let app = app.clone();
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();
    let ui = app.ui.clone();
    ui.upgrade_in_event_loop(move |ui| {
        ui.global::<ReleaseDetailsAdapter>().set_key(key.into());
        update_model(&app);
        ui.set_page(Page::ReleaseDetails);
    }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<ReleaseDetailsAdapter>().get_key();
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let release = Release::get(&library, &key).unwrap();
            let artists = release.artists(&library);
            let genres = release.genres(&library);
            let links = release.links(&library);
            let ui = app.ui.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut card: CardAdapter = release.clone().into();                
                card.image.image = app.images.lazy_get(release.clone(), 275, 275, |ui, image| {
                    let mut card = ui.global::<ReleaseDetailsAdapter>().get_card();
                    card.image.image = image;
                    ui.global::<ReleaseDetailsAdapter>().set_card(card);
                });

                let artists = artist_links(&artists);
                let genres = genre_links(&genres);
                let links: Vec<LinkAdapter> = links.iter().map(|link| {
                        LinkAdapter {
                            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                            url: link.url.clone().into(),
                        }
                    })
                    .collect();
    
                ui.global::<ReleaseDetailsAdapter>().set_card(card.into());
                ui.global::<ReleaseDetailsAdapter>().set_key(release.key.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_artists(ModelRc::from(artists.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                ui.global::<ReleaseDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<ReleaseDetailsAdapter>().set_dump(format!("{:?}", release).into());
    
            }).unwrap();
        });
    }).unwrap();
}

fn artist_links(artists: &[Artist]) -> Vec<LinkAdapter> {
    artists.iter().map(|artist| {
        LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}
