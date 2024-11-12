use std::{io::Cursor, sync::Arc};

use playback_rs::{Hint, Song};

use crate::{library::Library, model::{Model, Playlist}};

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
        // TODO maybe this is a metadata item?
        // TODO magic
        let key = format!("__dimple_system_play_queue_{}", self.library.id());
        let mut playlist = match self.library.get::<Playlist>(&key) {
            Some(play_queue) => play_queue,
            None => self.library.save(&Playlist {
                key: Some(key.to_string()),
                ..Default::default()
            })
        };
        playlist.hydrate(&self.library);
        playlist
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
        for track in play_queue.tracks {
            let content = Vec::<u8>::default();
            let sources = self.library.changelogs();
            let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();
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

