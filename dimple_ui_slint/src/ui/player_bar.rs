use std::time::Duration;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

use dimple_core::{model::Track, player::PlayerState};

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
        let _app = app.clone();
        std::thread::spawn(move || {
            loop {
                let app = _app.clone();
                _app.ui.upgrade_in_event_loop(move |ui| update_model(&app)).unwrap();
    
                // TODO magic
                std::thread::sleep(Duration::from_millis(100));
            }
    });
}
}

fn update_model(app: &App) {
    let player = app.player.clone();
    let current_track = player.current_queue_track().unwrap_or_default();
    let next_track = player.next_queue_track().unwrap_or_default();
    let _app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = _app.clone();
        let images = app.images.clone();

        let mut now_playing_recording = track_card(&current_track);
        now_playing_recording.image.image = images.lazy_get(current_track.clone(), 120, 120, |ui, image| {
            let mut adapter = ui.get_player_bar();
            adapter.now_playing_recording.image.image = image;
            ui.set_player_bar(adapter);
        });
    
        let adapter = crate::ui::PlayerBarAdapter {
            duration_seconds: player.track_duration().as_secs() as i32,
            duration_label: format_duration(&player.track_duration()).into(),
            position_seconds: player.track_position().as_secs() as i32,
            position_label: format_duration(&player.track_position()).into(),
            player_state: player.state().into(),
            now_playing_artist: artist_card(&current_track),
            up_next_artist: artist_card(&next_track),
            now_playing_release: release_card(&current_track),
            up_next_release: release_card(&next_track),
            now_playing_recording,
            up_next_recording: track_card(&next_track),
        };
    
        ui.set_player_bar(adapter);
    }).unwrap();
}

fn format_duration(dur: &Duration) -> String {
    format!("{}:{:02}", 
        dur.as_millis() / (60 * 1000), 
        dur.as_millis() % (60 * 1000) / 1000)
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

fn artist_card(track: &Track) -> CardAdapter {
    let track = track.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: track.artist.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: track.artist.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: "Artist".into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
    }
}

fn release_card(track: &Track) -> CardAdapter {
    let track = track.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: track.album.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", track.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: track.album.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: "Release".into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
    }
}

fn track_card(track: &Track) -> CardAdapter {
    let track = track.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: track.title.clone().unwrap_or_default().into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: "Track".into(),
            url: format!("dimple://track/{}", track.key.clone().unwrap_or_default()).into(),
        },
    }
}

