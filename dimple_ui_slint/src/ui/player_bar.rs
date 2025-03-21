use std::time::Duration;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::PlayerBarAdapter;

use dimple_core::model::Artist;
use dimple_core::model::Release;
use dimple_core::player::PlayerEvent;
use dimple_core::player::Song;
use dimple_core::{model::Track, player::PlayerState};

use super::app_window_controller::App;
use super::image_gen;
use super::images;

use slint::ComponentHandle;

#[derive(Clone)]
pub struct PlayerBar {
    app: App,
}

impl PlayerBar {
    pub fn new(app: &App) -> PlayerBar {
        let player_bar = Self {
            app: app.clone(),
        };

        player_bar.setup();

        player_bar
    }

    fn setup(&self) {
        // TODO move this stuff to app_controller.
        let app1 = self.app.clone();
        self.app.ui.upgrade_in_event_loop(move |ui| {
            let app = app1.clone();
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

        let player_bar = self.clone();
        self.app.player.notifier.observe(move |event| player_bar.on_player_event(&event));
    }

    fn on_player_event(&self, event: &PlayerEvent) {
        match &event {
            PlayerEvent::QueueIndex(_index) => self.update_model(),
            PlayerEvent::CurrentSong(song) => {
                self.update_model();
                self.update_waveform(song);
            },
            PlayerEvent::State(state) => {
                let state = state.clone();
                self.app.ui.upgrade_in_event_loop(move |ui| {
                    let adapter = ui.global::<PlayerBarAdapter>();
                    adapter.set_player_state(state.into());
                }).unwrap();
            },
            PlayerEvent::Duration(duration) => {
                let duration = duration.clone();
                self.app.ui.upgrade_in_event_loop(move |ui| {
                    let adapter = ui.global::<PlayerBarAdapter>();
                    adapter.set_duration_seconds(duration.as_secs() as i32);
                    adapter.set_duration_label(format_duration(&duration).into());
                }).unwrap();
            },
            PlayerEvent::Position(position) => {
                let position = position.clone();
                self.app.ui.upgrade_in_event_loop(move |ui| {
                    let adapter = ui.global::<PlayerBarAdapter>();
                    adapter.set_position_seconds(position.as_secs() as f32);
                    adapter.set_position_label(format_duration(&position).into());
                }).unwrap();
            },
        }
    }

    fn update_model(&self) {
        let player = self.app.player.clone();
        let library = self.app.library.clone();
        let images = self.app.images.clone();
        let current_track = player.current_queue_track().unwrap_or_default();
        let next_track = player.next_queue_track().unwrap_or_default();
        self.app.ui.upgrade_in_event_loop(move |ui| {
            let mut now_playing_track = track_card(&current_track);
            now_playing_track.image.image = images.lazy_get(current_track.clone(), 120, 120, |ui, image| {
                let mut card = ui.global::<PlayerBarAdapter>().get_now_playing_track();
                card.image.image = image;
                ui.global::<PlayerBarAdapter>().set_now_playing_track(card);
            });
    
            let mut up_next_track = track_card(&next_track);
            up_next_track.image.image = images.lazy_get(current_track.clone(), 120, 120, |ui, image| {
                let mut card = ui.global::<PlayerBarAdapter>().get_up_next_track();
                card.image.image = image;
                ui.global::<PlayerBarAdapter>().set_up_next_track(card);
            });
        
            let adapter: crate::ui::PlayerBarAdapter = ui.global::<PlayerBarAdapter>();
            adapter.set_duration_seconds(player.track_duration().as_secs() as i32);
            adapter.set_duration_label(format_duration(&player.track_duration()).into());
            adapter.set_position_seconds(player.track_position().as_secs() as f32);
            adapter.set_position_label(format_duration(&player.track_position()).into());
            adapter.set_player_state(player.state().into());
            adapter.set_now_playing_artist(artist_card(&current_track.artist(&library).unwrap_or_default()));
            adapter.set_now_playing_release(release_card(&current_track.release(&library).unwrap_or_default()));
            adapter.set_now_playing_track(now_playing_track);
            adapter.set_up_next_artist(artist_card(&next_track.artist(&library).unwrap_or_default()));
            adapter.set_up_next_release(release_card(&next_track.release(&library).unwrap_or_default()));
            adapter.set_up_next_track(up_next_track);
        }).unwrap();
    }
    
    fn update_waveform(&self, song: &Song) {
        let app = self.app.clone();
        let song = song.clone();
        std::thread::spawn(move || {
            let waveform = image_gen::gen_song_waveform(&song, 800, 32);
            // let waveform = image_gen::gen_song_spectrogram(&song, 800, 32);
            app.ui.upgrade_in_event_loop(move |ui| {
                let adapter: crate::ui::PlayerBarAdapter = ui.global::<PlayerBarAdapter>();
                adapter.set_waveform(images::dynamic_to_slint(&waveform));
            }).unwrap();
        });
    }
    
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

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
    }
}

fn release_card(release: &Release) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: release.date.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
        },
    }
}

fn track_card(track: &Track) -> CardAdapter {
    let track = track.clone();
    CardAdapter {
        key: track.key.clone().unwrap_or_default().into(),
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

