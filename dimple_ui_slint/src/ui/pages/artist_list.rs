fn artists(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    // std::thread::spawn(move || {
    //     let entity = Entities::Artist(Artist::default());
    //     let mut artists: Vec<Artist> = librarian.list(&entity, None)
    //         .filter_map(|e| match e {
    //             Entities::Artist(a) => Some(a),
    //             _ => None,
    //         })
    //         .collect();
    //     artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());
    //     let cards = artist_cards(artists, &librarian,
    //         Self::THUMBNAIL_WIDTH, 
    //         Self::THUMBNAIL_WIDTH);
    //     ui.upgrade_in_event_loop(move |ui| {
    //         let adapter = CardGridAdapter {
    //             cards: card_adapters(cards),
    //         };
    //         ui.set_card_grid_adapter(adapter);
    //         ui.set_page(0);
    //         log::info!("Rendering complete.");
    //     }).unwrap();
    // });
}

