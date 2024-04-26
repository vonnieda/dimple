// fn search(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
//     let url = url.to_owned();
//     // std::thread::spawn(move || {
//     //     ui.upgrade_in_event_loop(move |ui| {
//     //         ui.global::<Navigator>().set_busy(true);
//     //     }).unwrap();

//     //     let url = Url::parse(&url).unwrap();
//     //     let query = url.path_segments()
//     //         // TODO is this pattern wrong? Shouldn't the or be an error?
//     //         .ok_or("missing path").unwrap()
//     //         .nth(0)
//     //         .ok_or("missing query").unwrap();
//     //     let search_results: Vec<_> = librarian.search(query).collect();
//     //     // TODO woops, was sorting by name when they are returned by
//     //     // relevance. Once more sources are merged I'll need to bring
//     //     // rel to the front and sort on it.
//     //     // search_results.sort_by_key(|e| e.name().to_lowercase());
//     //     let cards = entity_cards(search_results, &librarian, 
//     //         Self::THUMBNAIL_WIDTH, 
//     //         Self::THUMBNAIL_WIDTH);
//     //     ui.upgrade_in_event_loop(move |ui| {
//     //         let adapter = CardGridAdapter {
//     //             cards: card_adapters(cards),
//     //         };
//     //         ui.set_card_grid_adapter(adapter);
//     //         ui.set_page(0);

//     //         ui.global::<Navigator>().set_busy(false);
//     //     }).unwrap();
//     // });
// }
