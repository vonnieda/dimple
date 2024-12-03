use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Track;
use slint::ComponentHandle;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::TrackDetailsAdapter;
use crate::ui::LinkAdapter;

pub fn track_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.library.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let mut track: Track = librarian.get(key).unwrap();

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

        let artists: Vec<Artist> = vec![];
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

            let mut links: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.into(),
                        url: link.into(),
                    }
                })
                .collect();
            // links.push(LinkAdapter {
            //     name: format!("dimple://recording/{}", track.recording.key.clone().unwrap_or_default()).into(),
            //     url: format!("dimple://recording/{}", track.recording.key.clone().unwrap_or_default()).into(),
            // });


            let mut adapter = TrackDetailsAdapter {
                card: track.clone().into(),
                artists: ModelRc::from(artists.as_slice()),                
                // summary: track.recording.summary.clone().unwrap_or_default().into(),
                genres: ModelRc::from(genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                // dump: format!("{}\n{}", 
                //     serde_json::to_string_pretty(&track).unwrap(),
                //     serde_json::to_string_pretty(&track.recording).unwrap()).into(),
                ..Default::default()
            };

            // adapter.card.image.image = images.lazy_get(track.model(), 275, 275, |ui, image| {
            //     let mut model = ui.get_track_details();
            //     model.card.image.image = image;
            //     ui.set_track_details(model);
            // });

            ui.set_track_details(adapter);
            ui.set_page(Page::TrackDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

