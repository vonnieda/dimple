use std::time::Duration;

use dimple_core::player::{Player, PlayerState};

use super::app_window_controller::App;

use slint::ComponentHandle;

pub fn player_bar_init(app: &App) {
    {
        let app = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let player = app.player.clone();
            ui.global::<crate::ui::AppState>().on_player_previous(move || player.previous());

            let player = app.player.clone();
            ui.global::<crate::ui::AppState>().on_player_play_pause(move || {
                if player.is_playing() { 
                    player.pause();
                }
                else {
                    player.play();
                }
            });

            let player = app.player.clone();
            ui.global::<crate::ui::AppState>().on_player_next(move || player.next());

            let player = app.player.clone();
            ui.global::<crate::ui::AppState>().on_player_seek(
                move |seconds| player.seek(Duration::from_secs(seconds as u64)));
        }).unwrap();
    }

    {
        let app = app.clone();
        std::thread::spawn(move || {
            loop {
                let player = app.player.clone();
                app.ui.upgrade_in_event_loop(move |ui| update_model(&player, &ui)).unwrap();
    
                // TODO magic
                std::thread::sleep(Duration::from_millis(100));
            }
        });
    }
}

fn format_duration(dur: &Duration) -> String {
    format!("{}:{:02}", 
        dur.as_millis() / (60 * 1000), 
        dur.as_millis() % (60 * 1000) / 1000)
}

fn update_model(player: &Player, ui: &crate::ui::AppWindow) {
    let current_track = player.current_queue_track().unwrap_or_default();
    let next_track = player.next_queue_track().unwrap_or_default();
    let adapter = crate::ui::PlayerBarAdapter {
        duration_seconds: player.track_duration().as_secs() as i32,
        duration_label: format_duration(&player.track_duration()).into(),
        position_seconds: player.track_position().as_secs() as i32,
        position_label: format_duration(&player.track_position()).into(),
        player_state: player.state().into(),
        now_playing_recording: current_track.into(),
        up_next_recording: next_track.into(),
        ..Default::default()
    };
    ui.set_player_bar(adapter);
}

impl From<PlayerState> for crate::ui::PlayerState {
    fn from(value: PlayerState) -> Self {
        match value {
            PlayerState::Stopped => crate::ui::PlayerState::Stopped,
            PlayerState::Playing => crate::ui::PlayerState::Playing,
            PlayerState::Paused => crate::ui::PlayerState::Paused,
        }            
    }
}

