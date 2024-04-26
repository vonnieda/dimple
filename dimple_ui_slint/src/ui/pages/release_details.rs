

// pub fn release_details(app: ) {
    // let url = url.to_owned();
    // std::thread::spawn(move || {
    //     ui.upgrade_in_event_loop(move |ui| {
    //         ui.global::<Navigator>().set_busy(true);
    //     }).unwrap();

    //     let url = Url::parse(&url).unwrap();
    //     let id = url.path_segments()
    //         .ok_or("missing path").unwrap()
    //         .nth(0)
    //         .ok_or("missing id").unwrap();

    //     let release = Release::get(id, librarian.as_ref())
    //         .ok_or("release not found").unwrap();
    //     let card = entity_card(&Entities::Release(release.clone()), 
    //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);

    //     let mut genres: Vec<_> = release.genres(librarian.as_ref())
    //         .map(|g| Link {
    //             name: g.name.unwrap_or_default(),
    //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
    //         })
    //         .collect();
    //     genres.sort_by_key(|g| g.name.to_owned());

    //     let mut artists: Vec<_> = release.artists(librarian.as_ref())
    //         .map(|a| Link {
    //             name: a.name.clone().unwrap_or_default(),
    //             url: format!("dimple://artist/{}", a.key.unwrap_or_default()),
    //         })
    //         .collect();
    //     artists.sort_by_key(|a| a.name.to_owned());

    //     let recordings: Vec<_> = release.recordings(librarian.as_ref()).collect();
    //     // TODO hmmmmm
    //     let medium = Medium {
    //         tracks: recordings.iter().map(|r| {
    //             let sources = r.sources(librarian.as_ref()).collect();
    //             Track {
    //                 title: r.title.str(),
    //                 length: r.length.unwrap_or_default(),
    //                 recording: r.clone(),
    //                 sources,
    //                 ..Default::default()
    //             }
    //         }).collect(),
    //         ..Default::default()
    //     };
    //     let media = vec![medium];

    //     ui.upgrade_in_event_loop(move |ui| {
    //         let model = ReleaseDetailsAdapter {                    
    //             card: card_adapter(&card),
    //             disambiguation: release.disambiguation.str().into(),
    //             genres: link_adapters(genres),
    //             summary: release.summary.str().into(),
    //             // primary_type: release.primary_type.str().into(),
    //             artists: link_adapters(artists),
    //             links: link_adapters(release_links(&release)),
    //             media: media_adapters(media),
    //             dump: serde_json::to_string_pretty(&release).unwrap().into(),
    //             ..Default::default()
    //         };
    //         ui.set_release_details(model);
    //         ui.set_page(3);
    //         ui.global::<Navigator>().set_busy(false);
    //     }).unwrap();
    // });
// }



// use dimple_core::model::Artist;
// use dimple_core::model::Entity;
// use dimple_core::model::Genre;
// use dimple_core::model::Model;
// use dimple_core::model::ReleaseGroup;
// use slint::ComponentHandle;
// use slint::Model as _;
// use slint::ModelRc;
// use url::Url;
// use crate::ui::app_window_controller::App;
// use crate::ui::Navigator;
// use crate::ui::Page;
// use crate::ui::ArtistDetailsAdapter;
// use dimple_core::db::Db;
// use crate::ui::CardAdapter;
// use crate::ui::LinkAdapter;

// pub fn artist_details(url: &str, app: &App) {
//     let url = url.to_owned();
//     let librarian = app.librarian.clone();
//     let ui = app.ui.clone();
//     let images = app.images.clone();
//     std::thread::spawn(move || {        
//         let url = Url::parse(&url).unwrap();
//         let key = url.path_segments().unwrap().nth(0).unwrap();

//         let artist: Artist = librarian.get(&Artist {
//             key: Some(key.to_string()),
//             ..Default::default()
//         }.into()).unwrap().unwrap().into();

//         let mut release_groups: Vec<ReleaseGroup> = librarian
//             .list(&ReleaseGroup::default().into(), Some(&Model::Artist(artist.clone())))
//             .unwrap()
//             .map(Into::into)
//             .collect();
//         release_groups.sort_by_key(|f| f.first_release_date.to_owned());
//         release_groups.reverse();

//         let genres: Vec<Genre> = librarian
//             .list(&Genre::default().into(), Some(&Model::Artist(artist.clone())))
//             .unwrap()
//             .map(Into::into)
//             .collect();

//         ui.upgrade_in_event_loop(move |ui| {
//             // TODO I hate all this duplication, but each one needs to filter on
//             // a different string, and then the closure needs to access a different
//             // field. So, duplication.
//             // TODO need to switch primary type to an enum
//             let albums: Vec<CardAdapter> = release_groups.iter().cloned()
//                 .filter(|release_group| release_group.primary_type == Some("album".to_string()))
//                 .enumerate()
//                 .map(|(index, release_group)| {
//                     let mut card: CardAdapter = release_group.clone().into();
//                     card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
//                         let mut card = ui.get_artist_details().albums.row_data(index).unwrap();
//                         card.image.image = image;
//                         ui.get_artist_details().albums.set_row_data(index, card);
//                     });
//                     card
//                 })
//                 .collect();
//             let eps: Vec<CardAdapter> = release_groups.iter().cloned()
//                 .filter(|release_group| release_group.primary_type == Some("ep".to_string()))
//                 .enumerate()
//                 .map(|(index, release_group)| {
//                     let mut card: CardAdapter = release_group.clone().into();
//                     card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
//                         let mut card = ui.get_artist_details().eps.row_data(index).unwrap();
//                         card.image.image = image;
//                         ui.get_artist_details().eps.set_row_data(index, card);
//                     });
//                     card
//                 })
//                 .collect();
//             let singles: Vec<CardAdapter> = release_groups.iter().cloned()
//                 .filter(|release_group| release_group.primary_type == Some("single".to_string()))
//                 .enumerate()
//                 .map(|(index, release_group)| {
//                     let mut card: CardAdapter = release_group.clone().into();
//                     card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
//                         let mut card = ui.get_artist_details().singles.row_data(index).unwrap();
//                         card.image.image = image;
//                         ui.get_artist_details().singles.set_row_data(index, card);
//                     });
//                     card
//                 })
//                 .collect();
//             let others: Vec<CardAdapter> = release_groups.iter().cloned()
//                 .filter(|release_group| release_group.primary_type != Some("album".to_string()) && release_group.primary_type != Some("ep".to_string()) && release_group.primary_type != Some("single".to_string()))
//                 .enumerate()
//                 .map(|(index, release_group)| {
//                     let mut card: CardAdapter = release_group.clone().into();
//                     card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
//                         let mut card = ui.get_artist_details().others.row_data(index).unwrap();
//                         card.image.image = image;
//                         ui.get_artist_details().others.set_row_data(index, card);
//                     });
//                     card
//                 })
//                 .collect();

//             let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
//                 LinkAdapter {
//                     name: genre.name.unwrap().into(),
//                     url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
//                 }
//             }).collect();

//             let links: Vec<LinkAdapter> = artist.links.iter().map(|link| {
//                 LinkAdapter {
//                     name: link.into(),
//                     url: link.into(),
//                 }
//             }).collect();

//             let mut adapter = ArtistDetailsAdapter {
//                 card: artist.clone().into(),
//                 disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
//                 summary: artist.summary.clone().unwrap_or_default().into(),
//                 albums: ModelRc::from(albums.as_slice()),
//                 singles: ModelRc::from(singles.as_slice()),
//                 eps: ModelRc::from(eps.as_slice()),
//                 others: ModelRc::from(others.as_slice()),
//                 genres: ModelRc::from(genres.as_slice()),
//                 links: ModelRc::from(links.as_slice()),
//                 dump: serde_json::to_string_pretty(&artist).unwrap().into(),
//                 ..Default::default()
//             };
//             adapter.card.image.image = images.get(artist.model(), 275, 275);
//             ui.set_artist_details(adapter);
//             ui.set_page(Page::ArtistDetails);
//             ui.global::<Navigator>().set_busy(false);
//         }).unwrap();
//     });
// }

