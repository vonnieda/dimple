use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Medium;
use dimple_core::model::Release;
use dimple_core::model::Track;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseDetailsAdapter;
use crate::ui::TrackAdapter;
use crate::ui::MediumAdapter;

pub fn release_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let release: Release = librarian.get2(Release {
            key: Some(key.to_string()),
            ..Default::default()
        }).unwrap();

        let mut artists: Vec<Artist> = librarian
            .list(&Artist::default().into(), &Some(release.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        artists.sort_by_key(|f| f.name.to_owned());

        let mut genres: Vec<Genre> = librarian
            .list(&Genre::default().into(), &Some(release.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        genres.sort_by_key(|f| f.name.to_owned());

        let media_and_tracks: Vec<_> = librarian
            .list(&Medium::default().into(), &Some(release.model()))
            .unwrap()
            .map(Into::<Medium>::into)
            .map(|medium| {
                let tracks: Vec<Track> = librarian.list(&Track::default().model(), &Some(medium.model()))
                    .unwrap()
                    .map(Into::<Track>::into)
                    .collect();
                (medium, tracks)
            })
            .collect();
        
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

            let links: Vec<LinkAdapter> = release.links.iter().map(|link| {
                LinkAdapter {
                    name: link.into(),
                    url: link.into(),
                }
            }).collect();

            let media: Vec<MediumAdapter> = media_and_tracks.iter().map(|(medium, tracks)| {
                MediumAdapter {
                    title: medium.title.clone().unwrap_or_default().into(),
                    tracks: track_adapters(tracks.to_vec()),
                }
            }).collect();

            let mut adapter = ReleaseDetailsAdapter {
                card: release.clone().into(),
                artists: ModelRc::from(artists.as_slice()),
                disambiguation: release.disambiguation.clone().unwrap_or_default().into(),
                summary: release.summary.clone().unwrap_or_default().into(),
                genres: ModelRc::from(genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                media: ModelRc::from(media.as_slice()),
                // primary_type: release.primary_type.clone().unwrap_or_default().into(),
                dump: serde_json::to_string_pretty(&release).unwrap().into(),                
                ..Default::default()
            };
            adapter.card.image.image = images.lazy_get(release.model(), 275, 275, |ui, image| {
                let mut model = ui.get_release_details();
                model.card.image.image = image;
                ui.set_release_details(model);
            });
            ui.set_release_details(adapter);
            ui.set_page(Page::ReleaseDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}


fn track_adapters(tracks: Vec<Track>) -> ModelRc<TrackAdapter> {
    let adapters: Vec<_> = tracks.iter()
        .map(|t| TrackAdapter {
            title: LinkAdapter {
                name: t.title.clone().unwrap_or_default().into(),
                url: format!("dimple://track/{}", t.key.clone().unwrap_or_default()).into(),
                ..Default::default()
            },
            track_number: format!("{}", t.number.clone().unwrap_or_default()).into(),
            length: length_to_string(t.length.clone().unwrap_or_default()).into(),
            // artists: Default::default(),
            // plays: 0,
            // source_count: t.sources.len() as i32,
            ..Default::default()
        })
        .collect();
    ModelRc::from(adapters.as_slice())
}

fn length_to_string(length: u32) -> String {
    format!("{}:{:02}", 
        length / (60 * 1000), 
        length % (60 * 1000) / 1000)
}

// fn media_adapters(media: Vec<Medium>) -> ModelRc<MediumAdapter> {
//     let adapters: Vec<_> = media.iter()
//         .map(|m| MediumAdapter {
//             title: format!("{} {} of {}", m.format, m.position, m.disc_count).into(),
//             tracks: track_adapters(m.tracks.clone()),
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

