

fn release_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
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
}
