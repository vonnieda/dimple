     
// let app = self.app.clone();
// self.ui.global::<AppState>().on_release_group_details_release_selected(
//     move |s| release_group_details::release_group_details_release_selected(&app, s.to_string()));

// // Load the sidebar
// let app = self.app.clone();
// std::thread::spawn(move || {
//     let mut pinned_items: Vec<Model> = vec![];
//     pinned_items.push(app.librarian.get2(Artist {
//         known_ids: KnownIds {
//             musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
//             ..Default::default()
//         },
//         ..Default::default()
//     }).unwrap().model());
//     pinned_items.push(app.librarian.get2(Artist {
//         known_ids: KnownIds {
//             musicbrainz_id: Some("65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab".to_string()),
//             ..Default::default()
//         },
//         ..Default::default()
//     }).unwrap().model());
//     pinned_items.push(app.librarian.get2(Artist {
//         known_ids: KnownIds {
//             musicbrainz_id: Some("c14b4180-dc87-481e-b17a-64e4150f90f6".to_string()),
//             ..Default::default()
//         },
//         ..Default::default()
//     }).unwrap().model());
//     pinned_items.push(app.librarian.get2(Artist {
//         known_ids: KnownIds {
//             musicbrainz_id: Some("69158f97-4c07-4c4e-baf8-4e4ab1ed666e".to_string()),
//             ..Default::default()
//         },
//         ..Default::default()
//     }).unwrap().model());
//     pinned_items.push(app.librarian.get2(Artist {
//         known_ids: KnownIds {
//             musicbrainz_id: Some("f1686ac4-3f28-4789-88eb-083ccb3a213a".to_string()),
//             ..Default::default()
//         },
//         ..Default::default()
//     }).unwrap().model());
//     let images = app.images.clone();
//     app.ui.upgrade_in_event_loop(move |ui| {
//         let cards: Vec<CardAdapter> = pinned_items.iter().cloned().enumerate()
//             .map(|(index, model)| {
//                 let mut card: CardAdapter = model_card(&model);
//                 card.image.image = images.lazy_get(model, 48, 48, move |ui, image| {
//                     let mut card = ui.get_sidebar().pinned_items.row_data(index).unwrap();
//                     card.image.image = image;
//                     ui.get_sidebar().pinned_items.set_row_data(index, card);
//                 });
//                 card
//             })
//             .collect();
//         let adapter = SideBarAdapter {
//             pinned_items: ModelRc::from(cards.as_slice()),
//         };
//         ui.set_sidebar(adapter);
//     }).unwrap();
// });


