fn settings(ui: slint::Weak<AppWindow>) {
    std::thread::spawn(move || {
        // TODO just playing around
        let cache_stats = vec![
            "Metadata Items: 5276 / 27.3MB",
            "Tracks: 986 / 36.2GB",
            "Images: 1286 / 12.6GB",
        ];
        
        ui.upgrade_in_event_loop(move |ui| {
            let cache_stats: Vec<SharedString> = cache_stats.into_iter()
                .map(Into::into)
                .collect();
            let adapter = SettingsAdapter {
                cache_stats: ModelRc::from(cache_stats.as_slice()),
            };
            ui.set_settings_adapter(adapter);
            ui.set_page(6)
        }).unwrap();
    });
}

// let ui = self.ui.as_weak();
// let librarian = self.librarian.clone();
// // TODO moves to settings, or side bar, or wherever it's supposed to go.
// self.ui.global::<AppState>().on_set_online(move |online| {
//     let librarian = librarian.clone();
//     ui.upgrade_in_event_loop(move |ui| {
//         let librarian = librarian.clone();
//         librarian.set_access_mode(if online { &AccessMode::Online } else { &AccessMode::Offline });
//         ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
//     }).unwrap();
// });

