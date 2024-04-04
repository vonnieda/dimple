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

