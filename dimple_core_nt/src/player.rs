use std::sync::Arc;

use playback_rs::Song;

use crate::{library::{Library, LibraryModel}, model::Playlist};

pub struct Player {
    library: Arc<Library>,
}

impl Player {
    pub fn new(library: Arc<Library>) -> Player {
        Player {
            library,
        }
    }

    pub fn play_queue(&self) -> Playlist {
        const KEY: &str = "__dimple_system_play_queue";
        let play_queue = Playlist::get(&self.library, KEY);
        match play_queue {
            Some(play_queue) => play_queue,
            None => Playlist {
                key: Some(KEY.to_string()),
                ..Default::default()
            }.save(&self.library)
        }
    }

    pub fn play_queue_add(&self, track_key: &str) {
        let playlist = self.play_queue();
        self.library.playlist_add(&playlist, track_key);
    }

    pub fn play_queue_clear(&self) {
        let playlist = self.play_queue();
        self.library.playlist_clear(&playlist);
    }

    pub fn play(&self) {
        let player = playback_rs::Player::new(None).unwrap();
        let play_queue = self.play_queue();
        let filenames = play_queue.tracks.iter().map(|track| track.path.clone());
        for next_song in filenames {
            let song = Song::from_file(&next_song, None).unwrap();
            while player.has_next_song() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            player.play_song_next(&song, None).unwrap();
        }
        while player.has_current_song() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}