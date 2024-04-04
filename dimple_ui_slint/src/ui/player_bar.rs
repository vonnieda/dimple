        // // TODO moves to player_bar
        // let ui = self.ui.as_weak();
        // let player = self.player.clone();
        // thread::spawn(move || {
        //     ui.upgrade_in_event_loop(move |ui| {
        //         fn length_to_string(length: u32) -> String {
        //             format!("{}:{:02}", 
        //                 length / (60 * 1000), 
        //                 length % (60 * 1000) / 1000)
        //         }
                
        //         let adapter = PlayerBarAdapter {
        //             duration_seconds: player.duration().as_secs() as i32,
        //             duration_label: length_to_string(player.duration().as_secs() as u32).into(),
        //             position_seconds: player.position().as_secs() as i32,
        //             position_label: length_to_string(player.position().as_secs() as u32).into(),
        //             ..Default::default()
        //         };
        //         ui.set_player_bar_adapter(adapter);
        //     }).unwrap();
        //     thread::sleep(Duration::from_millis(100));
        // });
