use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::ArtistDetailsAdapter;
use dimple_core::db::Db;
use crate::ui::CardAdapter;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseGroupDetailsAdapter;

pub fn release_group_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let release_group: ReleaseGroup = librarian.get(&ReleaseGroup {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        let mut genres: Vec<Genre> = librarian
            .list(&Genre::default().into(), Some(&release_group.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        genres.sort_by_key(|f| f.name.to_owned());

        let mut releases: Vec<Release> = librarian
            .list(&Release::default().into(), Some(&release_group.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        releases.sort_by_key(|f| f.date.to_owned());
        releases.reverse();

        ui.upgrade_in_event_loop(move |ui| {
            // let others: Vec<CardAdapter> = release_groups.iter().cloned()
            //     .filter(|release_group| release_group.primary_type != Some("album".to_string()) && release_group.primary_type != Some("ep".to_string()) && release_group.primary_type != Some("single".to_string()))
            //     .enumerate()
            //     .map(|(index, release_group)| {
            //         let mut card: CardAdapter = release_group.clone().into();
            //         card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
            //             let mut card = ui.get_artist_details().others.row_data(index).unwrap();
            //             card.image.image = image;
            //             ui.get_artist_details().others.set_row_data(index, card);
            //         });
            //         card
            //     })
            //     .collect();

            let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
                LinkAdapter {
                    name: genre.name.unwrap().into(),
                    url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
                }
            }).collect();

            let links: Vec<LinkAdapter> = release_group.links.iter().map(|link| {
                LinkAdapter {
                    name: link.into(),
                    url: link.into(),
                }
            }).collect();

            let mut adapter = ReleaseGroupDetailsAdapter {
                card: release_group.clone().into(),
                disambiguation: release_group.disambiguation.clone().unwrap_or_default().into(),
                summary: release_group.summary.clone().unwrap_or_default().into(),
                // albums: ModelRc::from(albums.as_slice()),
                // singles: ModelRc::from(singles.as_slice()),
                // eps: ModelRc::from(eps.as_slice()),
                // others: ModelRc::from(others.as_slice()),
                genres: ModelRc::from(genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                dump: serde_json::to_string_pretty(&release_group).unwrap().into(),
                ..Default::default()
            };
            adapter.card.image.image = images.get(release_group.model(), 275, 275);
            ui.set_release_group_details(adapter);
            ui.set_page(Page::ReleaseGroupDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}




// fn release_group_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
//     let url = url.to_owned();
//     // std::thread::spawn(move || {
//     //     ui.upgrade_in_event_loop(move |ui| {
//     //         ui.global::<Navigator>().set_busy(true);
//     //     }).unwrap();

//     //     let url = Url::parse(&url).unwrap();
//     //     let id = url.path_segments()
//     //         .ok_or("missing path").unwrap()
//     //         .nth(0)
//     //         .ok_or("missing id").unwrap();
//     //     let release_group = ReleaseGroup::get(id, librarian.as_ref())
//     //         .ok_or("release group not found").unwrap();
//     //     let card = entity_card(&Entities::ReleaseGroup(release_group.clone()), 
//     //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
//     //     let mut genres: Vec<_> = release_group.genres(librarian.as_ref())
//     //         .map(|g| Link {
//     //             name: g.name.unwrap_or_default(),
//     //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
//     //         })
//     //         .collect();
//     //     genres.sort_by_key(|g| g.name.to_owned());
//     //     let mut artists: Vec<_> = release_group.artists(librarian.as_ref())
//     //         .map(|a| Link {
//     //             name: a.name.clone().unwrap_or_default(),
//     //             url: format!("dimple://artist/{}", a.key.unwrap_or_default()),
//     //         })
//     //         .collect();
//     //     artists.sort_by_key(|a| a.name.to_owned());
//     //     let mut releases: Vec<_> = release_group.releases(librarian.as_ref()).collect();
//     //     releases.sort_by_key(|r| r.date.clone());
//     //     let release_cards = release_cards(releases, &librarian, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT);

//     //     ui.upgrade_in_event_loop(move |ui| {
//     //         let model = ReleaseGroupDetailsAdapter {                    
//     //             card: card_adapter(&card),
//     //             disambiguation: release_group.disambiguation.str().into(),
//     //             genres: link_adapters(genres),
//     //             summary: release_group.summary.str().into(),
//     //             primary_type: release_group.primary_type.str().into(),
//     //             artists: link_adapters(artists),
//     //             links: link_adapters(release_group_links(&release_group)),
//     //             // media: media_adapters(release.media),
//     //             releases: card_adapters(release_cards),
//     //             dump: serde_json::to_string_pretty(&release_group).unwrap().into(),
//     //             ..Default::default()
//     //         };
//     //         ui.set_release_group_details(model);
//     //         ui.set_page(2);
//     //         ui.global::<Navigator>().set_busy(false);
//     //     }).unwrap();
//     // });
// }
