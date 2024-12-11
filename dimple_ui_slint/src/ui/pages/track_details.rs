use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Track;
use dimple_core::services::lrclib::LrclibService;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
use crate::ui::LinkAdapter;
use slint::ComponentHandle as _;
use crate::ui::CardAdapter;

pub fn track_details_init(app: &App) {
    // _app.library.on_change(|library, type_name, key| println!("{} {}", type_name, key));
    // let app = _app.clone();
    // _app.library.on_change(move |library, model_type, key| {       
    //     let app = app.clone();
    //     let model_type = model_type.to_string();
    //     let ui = app.ui.clone();
    //     ui.upgrade_in_event_loop(move |ui| {
    //         if ui.get_page() == Page::TrackDetails && model_type == "Track" {
    //             app.refresh();
    //         }
    //     });
    // });
    let _app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = _app.clone();
        ui.global::<TrackDetailsAdapter>().on_play_now(move |key| play_now(&app, &key));
        let app = _app.clone();
        ui.global::<TrackDetailsAdapter>().on_add_to_queue(move |key| add_to_queue(&app, &key));
    }).unwrap();
}

pub fn track_details(url: &str, app: &App) {
    let url = url.to_owned();
    let library = app.library.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        // TODO temp for testing, not sure where this is gonna live yet.
        let track = library.get::<Track>(key).unwrap();
        {
            let track = track.clone();
            let library = library.clone();
            std::thread::spawn(move || {
                let lrclib = LrclibService {};
                lrclib.track_lyrics(&library, &track);
            });
        }

        // track.recording = librarian.list(&Recording::default().model(), &Some(track.model()))
        //     .unwrap().map(Into::<Recording>::into).next().unwrap();

        // track.recording.genres = librarian
        //     .list(&Genre::default().into(), &Some(track.recording.model()))
        //     .unwrap().map(Into::into).collect();

        // let mut artists: Vec<Artist> = librarian
        //     .list(&Artist::default().into(), &Some(track.recording.model()))
        //     .unwrap().map(Into::into).collect();
        // artists.sort_by_key(|f| f.name.to_owned());

        // track.genres = librarian
        //     .list(&Genre::default().into(), &Some(track.model()))
        //     .unwrap().map(Into::into).collect();
        // track.genres.sort_by_key(|genre| genre.name.clone().unwrap_or_default().to_lowercase());

        let artists: Vec<Artist> = vec![ Artist {
            // TODO wrong key, just for testing.
            key: track.key.clone(),
            name: track.artist.clone(),
            ..Default::default()
        }];
        let genres: Vec<Genre> = vec![];
        let links: Vec<String> = vec![];

        ui.upgrade_in_event_loop(move |ui| {
            let artists: Vec<LinkAdapter> = artists.iter().cloned().map(|artist| {
                LinkAdapter {
                    name: artist.name.unwrap().into(),
                    url: format!("dimple://artist/{}", artist.key.unwrap()).into(),
                }
            }).collect();

            let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
                LinkAdapter {
                    name: genre.name.unwrap().into(),
                    url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
                }
            }).collect();

            let links: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.into(),
                        url: link.into(),
                    }
                })
                .collect();

            let mut card: CardAdapter = track.clone().into();
            card.image.image = images.lazy_get(track.clone(), 275, 275, |ui, image| {
                let mut card = ui.global::<TrackDetailsAdapter>().get_card();
                card.image.image = image;
                ui.global::<TrackDetailsAdapter>().set_card(card);
            });

            ui.global::<TrackDetailsAdapter>().set_card(card.into());
            ui.global::<TrackDetailsAdapter>().set_key(track.key.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_artists(ModelRc::from(artists.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_summary(track.summary.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_disambiguation(track.disambiguation.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_genres(ModelRc::from(genres.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_lyrics(track.lyrics.clone().unwrap_or_default().into());
            ui.global::<TrackDetailsAdapter>().set_links(ModelRc::from(links.as_slice()));
            ui.global::<TrackDetailsAdapter>().set_dump(format!("{:?}", track).into());

            ui.set_page(Page::TrackDetails);
        }).unwrap();
    });
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

