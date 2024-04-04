fn release_group_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
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
    //     let release_group = ReleaseGroup::get(id, librarian.as_ref())
    //         .ok_or("release group not found").unwrap();
    //     let card = entity_card(&Entities::ReleaseGroup(release_group.clone()), 
    //         Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
    //     let mut genres: Vec<_> = release_group.genres(librarian.as_ref())
    //         .map(|g| Link {
    //             name: g.name.unwrap_or_default(),
    //             url: format!("dimple://genre/{}", g.key.unwrap_or_default()),
    //         })
    //         .collect();
    //     genres.sort_by_key(|g| g.name.to_owned());
    //     let mut artists: Vec<_> = release_group.artists(librarian.as_ref())
    //         .map(|a| Link {
    //             name: a.name.clone().unwrap_or_default(),
    //             url: format!("dimple://artist/{}", a.key.unwrap_or_default()),
    //         })
    //         .collect();
    //     artists.sort_by_key(|a| a.name.to_owned());
    //     let mut releases: Vec<_> = release_group.releases(librarian.as_ref()).collect();
    //     releases.sort_by_key(|r| r.date.clone());
    //     let release_cards = release_cards(releases, &librarian, Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT);

    //     ui.upgrade_in_event_loop(move |ui| {
    //         let model = ReleaseGroupDetailsAdapter {                    
    //             card: card_adapter(&card),
    //             disambiguation: release_group.disambiguation.str().into(),
    //             genres: link_adapters(genres),
    //             summary: release_group.summary.str().into(),
    //             primary_type: release_group.primary_type.str().into(),
    //             artists: link_adapters(artists),
    //             links: link_adapters(release_group_links(&release_group)),
    //             // media: media_adapters(release.media),
    //             releases: card_adapters(release_cards),
    //             dump: serde_json::to_string_pretty(&release_group).unwrap().into(),
    //             ..Default::default()
    //         };
    //         ui.set_release_group_details(model);
    //         ui.set_page(2);
    //         ui.global::<Navigator>().set_busy(false);
    //     }).unwrap();
    // });
}
