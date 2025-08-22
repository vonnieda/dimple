// // Image "service": Downloads images for new artists, releases, etc.
// // We'll want this to run periodically for any artists with no art
// // and whenever a new artist is added.
// let app_clone = app.clone();
// std::thread::spawn(move || {
// let app = app_clone;
// let rx = app.library.db.subscribe();
// loop {
//     if let Ok(event) = rx.recv() {
//         if let DbEvent::Insert(entity_type, entity_id) = event {
//             if entity_type == "Artist" {
//                 if let Ok(Some(artist)) = app.library.db.get::<Artist>(&entity_id) {
//                     // TODO do something cool
//                     println!("new artist {:?}", artist.name);
//                 }
//             }
//         }
//     }
// }
// });
