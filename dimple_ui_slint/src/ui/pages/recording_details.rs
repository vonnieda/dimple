fn recording_details(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    //     let url = url.to_owned();
    //     std::thread::spawn(move || {
    //         let url = Url::parse(&url).unwrap();
    //         let id = url.path_segments()
    //             .ok_or("missing path").unwrap()
    //             .nth(0)
    //             .ok_or("missing id").unwrap();
    //         let recording = Recording::get(id, librarian.as_ref())
    //             .ok_or("recording not found").unwrap();
    //         let card = entity_card(&Entities::Recording(recording.clone()),
    //             Self::THUMBNAIL_WIDTH, Self::THUMBNAIL_HEIGHT, &librarian);
    //         let genres = recording.genres.iter()
    //             .map(|g| Link {
    //                 name: g.name.clone(),
    //                 url: format!("dimple://genre/{}", g.name),
    //             })
    //             .collect();
    //         let artists = recording.artist_credits.iter()
    //             .map(|a| Link {
    //                 name: a.name.clone().unwrap_or_default(),
    //                 url: format!("dimple://artist/{}", a.key),
    //             })
    //             .collect();
    //         let isrcs = recording.isrcs.iter()
    //             .map(|i| Link {
    //                 name: i.to_string(),
    //                 url: format!("https://api.deezer.com/2.0/track/isrc:{}", i),
    //             })
    //             .collect();
    //         // let releases: Vec<_> = release_group.releases.clone();
    //         // let release_cards = release_cards(releases, &librarian, 500, 500);
    //         // let release = release_group.releases.first()
    //         //     .ok_or("no releases")
    //         //     .unwrap();
    //         // let release = release.fetch(librarian.as_ref())
    //         //     .ok_or("release not found")
    //         //     .unwrap();

    //         ui.upgrade_in_event_loop(move |ui| {
    //             let model = RecordingDetailsAdapter {                    
    //                 card: card_adapter(&card),
    //                 disambiguation: recording.disambiguation.clone().into(),
    //                 genres: link_adapters(genres),
    //                 summary: recording.summary.clone().into(),
    //                 // primary_type: recording.primary_type.clone().into(),
    //                 artists: link_adapters(artists),
    //                 links: link_adapters(recording_links(&recording)),
    //                 isrcs: link_adapters(isrcs),
    //                 // media: media_adapters(release.media),
    //                 // releases: card_adapters(release_cards),
    //                 // releases: Default::default()
    //             };
    //             ui.set_recording_details(model);
    //             ui.set_page(4)
    //         }).unwrap();
    //     });
    // }
}
