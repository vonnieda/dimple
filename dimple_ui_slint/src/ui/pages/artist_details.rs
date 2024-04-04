fn artist_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let url = url.to_owned();
    // std::thread::spawn(move || {
    //     ui.upgrade_in_event_loop(move |ui| {
    //         ui.global::<Navigator>().set_busy(true);
    //     }).unwrap();

    //     let url = Url::parse(&url).unwrap();
    //     let id = url.path_segments()
    //         .ok_or("missing path").unwrap()
    //         .nth(0)
    //         .ok_or("missing id").unwrap();
    //     let artist = Artist::get(id, librarian.as_ref()).unwrap();
    //     let card = entity_card(&Entities::Artist(artist.clone()), 
    //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
    //     let mut release_groups: Vec<_> = artist.release_groups(librarian.as_ref()).collect();
    //     release_groups.sort_by_key(|f| f.first_release_date.to_owned());
    //     release_groups.reverse();
    //     let release_group_cards: Vec<_> = release_groups.par_iter()
    //         .map(|rg| (rg.primary_type.str().to_lowercase().clone(), 
    //             release_group_card(rg, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian)))
    //         .collect();
    //     let album_cards: Vec<_> = release_group_cards.par_iter()
    //         .filter(|(primary_type, _card)| primary_type == "album")
    //         .map(|(_primary_type, card)| card.clone())
    //         .collect();
    //     let single_cards: Vec<_> = release_group_cards.par_iter()
    //         .filter(|(primary_type, _card)| primary_type == "single")
    //         .map(|(_primary_type, card)| card.clone())
    //         .collect();
    //     let ep_cards: Vec<_> = release_group_cards.par_iter()
    //         .filter(|(primary_type, _card)| primary_type == "ep")
    //         .map(|(_primary_type, card)| card.clone())
    //         .collect();
    //     let other_release_group_cards: Vec<_> = release_group_cards.par_iter()
    //         .filter(|(primary_type, _card)| primary_type != "album" && primary_type != "single" && primary_type != "ep")
    //         .map(|(_primary_type, card)| card.clone())
    //         .collect();
    //     let genres: Vec<_> = artist.genres(librarian.as_ref())
    //         .map(|g| Link {
    //             name: g.name.unwrap_or_default(),
    //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
    //         })
    //         .collect();

    //     ui.upgrade_in_event_loop(move |ui| {
    //         let adapter = ArtistDetailsAdapter {
    //             card: card_adapter(&card),
    //             disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
    //             summary: artist.summary.clone().unwrap_or_default().into(),
    //             albums: card_adapters(album_cards),
    //             singles: card_adapters(single_cards),
    //             eps: card_adapters(ep_cards),
    //             others: card_adapters(other_release_group_cards),
    //             genres: link_adapters(genres),
    //             links: link_adapters(artist_links(&artist)),
    //             dump: serde_json::to_string_pretty(&artist).unwrap().into(),
    //             ..Default::default()
    //         };
    //         ui.set_artist_details(adapter);
    //         ui.set_page(1);
    //         ui.global::<Navigator>().set_busy(false);
    //     }).unwrap();
    // });
}
