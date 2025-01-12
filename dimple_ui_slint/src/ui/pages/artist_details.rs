use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use slint::ComponentHandle as _;
use slint::ModelRc;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ArtistDetailsAdapter;
use crate::ui::ImageLinkAdapter;

pub fn artist_details_init(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app1.clone();
        ui.global::<ArtistDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = app1.clone();
        ui.global::<ArtistDetailsAdapter>().on_add_to_queue(move |key| add_to_queue(&app, &key));
    }).unwrap();

    
    // TODO filter events
    let app1 = app.clone();
    app.library.on_change(Box::new(move |_event| update_model(&app1)));
}

pub fn artist_details(url: &str, app: &App) {
    let app = app.clone();
    let url = Url::parse(&url).unwrap();
    let key = url.path_segments().unwrap().nth(0).unwrap().to_string();
    let ui = app.ui.clone();
    ui.upgrade_in_event_loop(move |ui| {
        ui.global::<ArtistDetailsAdapter>().set_key(key.into());
        update_model(&app);
        ui.set_page(Page::ArtistDetails);
    }).unwrap();
}

fn play_now(app: &App, key: &str) {
    // let app = app.clone();
    // let key = key.to_string();
    // app.ui.upgrade_in_event_loop(move |ui| {
    //     // TODO think about ephemeral or secondary playlist, or even
    //     // a playlist inserted inbetween the playing items
    //     let play_queue = app.player.queue();
    //     app.library.playlist_add(&play_queue, &key);
    //     let len = play_queue.len(&app.library);
    //     app.player.set_queue_index(len - 1);
    //     app.player.play();
    // }).unwrap();
    todo!()
}

fn add_to_queue(app: &App, key: &str) {
    // let app = app.clone();
    // let key = key.to_string();
    // app.ui.upgrade_in_event_loop(move |ui| {
    //     let play_queue = app.player.queue();
    //     let release = Artist::get(&app.library, &key).unwrap();
    //     for track in release.tracks(&app.library) {
    //         app.library.playlist_add(&play_queue, &track.key.unwrap());
    //     }
    //     app.player.play();
    // }).unwrap();
}

fn update_model(app: &App) {
    let app1 = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.global::<ArtistDetailsAdapter>().get_key();
        if key.is_empty() {
            return
        }
        let library = app1.library.clone();
        let app = app1.clone();
        std::thread::spawn(move || {
            let artist = Artist::get(&library, &key).unwrap();
            let genres = artist.genres(&library);
            let links = artist.links(&library);
            let releases = artist.releases(&app.library);
            let ui = app.ui.clone();
            let images = app.images.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut card: CardAdapter = artist.clone().into();                
                // card.image.image = app.images.lazy_get(release.clone(), 275, 275, |ui, image| {
                //     let mut card = ui.global::<ArtistDetailsAdapter>().get_card();
                //     card.image.image = image;
                //     ui.global::<ArtistDetailsAdapter>().set_card(card);
                // });

                let genres = genre_links(&genres);
                let links: Vec<LinkAdapter> = links.iter().map(|link| {
                        LinkAdapter {
                            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                            url: link.url.clone().into(),
                        }
                    })
                    .collect();

                let releases = release_cards(&images, &releases, &library);
                ui.global::<ArtistDetailsAdapter>().set_card(card.into());
                ui.global::<ArtistDetailsAdapter>().set_key(artist.key.clone().unwrap_or_default().into());
                ui.global::<ArtistDetailsAdapter>().set_releases(ModelRc::from(releases.as_slice()));
                ui.global::<ArtistDetailsAdapter>().set_summary(artist.summary.clone().unwrap_or_default().into());
                ui.global::<ArtistDetailsAdapter>().set_disambiguation(artist.disambiguation.clone().unwrap_or_default().into());
                ui.global::<ArtistDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
                ui.global::<ArtistDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
                ui.global::<ArtistDetailsAdapter>().set_dump(format!("{:?}", artist).into());
            }).unwrap();
        });
    }).unwrap();
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            // card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //     let mut card = ui.get_release_list().cards.row_data(index).unwrap();
            //     card.image.image = image;
            //     ui.get_release_list().cards.set_row_data(index, card);
            // });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
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
            name: format!("{} {}", 
                release.date.unwrap_or_default(), 
                release.release_group_type.unwrap_or_default()).into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

