use std::{collections::HashMap, io::Cursor, sync::{mpsc::{self, Receiver, Sender}, Arc, RwLock}, time::Duration};

use playback_rs::{Hint, Song};

use crate::{library::Library, model::{Model, Playlist, Track, TrackSource}};

#[derive(Clone)]
pub struct Player {
    library: Arc<Library>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
}

impl Player {
    pub fn new(library: Arc<Library>) -> Player {
        let (sender, receiver) = std::sync::mpsc::channel();
        let player = Player {
            library,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
        };
        // TODO library.on_change() and watch for changes to the playlist, which
        // will cause us to need to reevaulate if the right song is loaded.
        // TODO refactor this as InnerPlayer so that we can keep the state
        // that is part of the thread contained
        {
            let player = player.clone();
            std::thread::spawn(move || player.player_worker(receiver));
        }
        player
    }

    pub fn play(&self) {
        self.sender.send(PlayerCommand::Play).unwrap();
    }

    pub fn pause(&self) {
        self.sender.send(PlayerCommand::Pause).unwrap();
    }

    pub fn next(&self) {
        if let Ok(mut shared_state) = self.shared_state.write() {
            shared_state.queue_index = (shared_state.queue_index + 1).min(self.queue().len(&self.library) - 1);
        }
    }

    pub fn previous(&self) {
        if let Ok(mut shared_state) = self.shared_state.write() {
            const REWIND_SECONDS: u64 = 3;
            if shared_state.track_position.as_secs() < REWIND_SECONDS {
                shared_state.queue_index = (shared_state.queue_index - 1).max(0);
            }
            else {
                self.sender.send(PlayerCommand::Seek(Duration::ZERO)).unwrap();
            }
        }
    }

    pub fn seek(&self, position: Duration) {
        self.sender.send(PlayerCommand::Seek(position)).unwrap();
    }

    pub fn track_duration(&self) -> Duration {
        self.shared_state.read().unwrap().track_duration
    }

    pub fn track_position(&self) -> Duration {
        self.shared_state.read().unwrap().track_position
    }

    pub fn state(&self) -> PlayerState {
        self.shared_state.read().unwrap().inner_player_state.clone()
    }

    pub fn is_playing(&self) -> bool {
        self.state() == PlayerState::Playing
    }

    /// TODO magic. maybe this is a metadata item?
    /// Starting to think maybe this goes away completely and we have
    /// set_play_queue(Playlist) which copies in the tracks and
    /// such so that that metadata is always available. The play queue
    /// key can live in Library, and I suppose Library::default_play_queue()
    pub fn queue(&self) -> Playlist {
        let key = format!("__dimple_system_play_queue_{}", self.library.id());
        let playlist = match self.library.get::<Playlist>(&key) {
            Some(play_queue) => play_queue,
            None => self.library.save(&Playlist {
                key: Some(key.to_string()),
                ..Default::default()
            })
        };
        playlist
    }

    // TODO woops need playlistitem instead of track cause if a track is right after itself it
    // gets skipped cause next track appears already loaded. So we need the
    // playlist item which includes the ordinal, or at least has a different key so
    // not equal.
    fn player_worker(&self, receiver: Receiver<PlayerCommand>) {
        let inner = playback_rs::Player::new(None).unwrap();
        inner.set_playing(false);
        loop {
            while let Ok(command) = receiver.recv_timeout(Duration::from_millis(100)) {
                match command {
                    PlayerCommand::Play => inner.set_playing(true),
                    PlayerCommand::Pause => inner.set_playing(false),
                    PlayerCommand::Seek(position) => {inner.seek(position);},
                }
            }

            // If the player has no next song set, but we had set one previously
            // then the current song finished and the next song is now playing.
            // We need to advance the queue and update the state so that it
            // reflects which song is now loaded.
            if !inner.has_next_song() && self.shared_state.read().unwrap().next_loaded_track.is_some() {
                log::warn!("song finished, advancing the queue!");
                if let Ok(mut shared_state) = self.shared_state.write() {
                    // TODO detect end of queue and send pause or stop
                    shared_state.current_loaded_track = shared_state.next_loaded_track.clone();
                    shared_state.next_loaded_track = None;
                }
                self.next();
            }

            if let Some(current_queue_track) = self.current_queue_track() {
                if let Ok(mut shared_state) = self.shared_state.write() {
                    if shared_state.current_loaded_track != Some(current_queue_track.clone()) {
                        log::warn!("current track mismatch, loading current track");
                        inner.play_song_now(&self.get_song(&current_queue_track), None).unwrap();
                        shared_state.current_loaded_track = Some(current_queue_track.clone());
                    }
                }
            }

            if self.state() == PlayerState::Playing {
                if let Some(next_queue_track) = self.next_queue_track() {
                    if let Ok(mut shared_state) = self.shared_state.write() {
                        if shared_state.next_loaded_track != Some(next_queue_track.clone()) {
                            log::warn!("next track mismatch, loading next track");
                            inner.play_song_next(&self.get_song(&next_queue_track), None).unwrap();
                            shared_state.next_loaded_track = Some(next_queue_track.clone());
                        }
                    }
                }
            }

            let (position, duration) = inner.get_playback_position().unwrap_or_default();
            if let Ok(mut shared_state) = self.shared_state.write() {
                shared_state.track_position = position;
                shared_state.track_duration = duration;
                shared_state.inner_player_state = match (inner.is_playing(), inner.has_current_song()) {
                    (false, false) => PlayerState::Stopped,
                    (false, true) => PlayerState::Paused,
                    (true, false) => PlayerState::Stopped,
                    (true, true) => PlayerState::Playing,                    
                };
            }
        }
    }

    fn get_song(&self, track: &Track) -> Song {
        let content = self.library.load_track_content(track).expect("No valid sources found.");
        let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();            
        song
    }

    pub fn current_queue_index(&self) -> usize {
        self.shared_state.read().unwrap().queue_index
    }

    pub fn current_queue_track(&self) -> Option<Track> {
        if let Ok(state) = self.shared_state.read() {
            if state.queue_index >= self.queue().len(&self.library) {
                return None;
            }
            Some(self.queue().tracks(&self.library)[state.queue_index].clone())
        }
        else {
            None
        }
    }

    pub fn next_queue_track(&self) -> Option<Track> {
        if let Ok(state) = self.shared_state.read() {
            if state.queue_index + 1 >= self.queue().len(&self.library) {
                return None;
            }
            Some(self.queue().tracks(&self.library)[state.queue_index + 1].clone())
        }
        else {
            None
        }
    }
}

struct SongCache {
    songs_by_track_key: Arc<RwLock<HashMap<String, Song>>>,
}

impl SongCache {
    
}

#[derive(Default)]
struct SharedState {
    pub queue_index: usize,
    pub track_duration: Duration,
    pub track_position: Duration,
    pub inner_player_state: PlayerState,
    pub current_loaded_track: Option<Track>,
    pub next_loaded_track: Option<Track>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Debug)]
enum PlayerCommand {
    Play,
    Pause,
    Seek(Duration),
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::{Duration, Instant}};

    use crate::{library::Library, player::PlayerState};

    use super::Player;

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let library = Arc::new(Library::open("file:ee2e5b97-b997-431d-8224-d361e905d071?mode=memory&cache=shared"));
        let player = Player::new(library.clone());
        library.import("tests/data/media_files");
        let tracks = library.tracks();
        let play_queue = player.queue();

        std::thread::sleep(Duration::from_secs(5));
        for track in &tracks[0..3] {
            library.playlist_add(&play_queue, track.key.as_ref().unwrap());
        }

        std::thread::sleep(Duration::from_secs(5));
        player.play();

        std::thread::sleep(Duration::from_secs(5));
        // let t = Instant::now();
        // loop {
        //     std::thread::sleep(Duration::from_secs(1));
        // }
        // while player.state() == PlayerState::Playing {}
        // assert!(t.elapsed().as_secs() > 5);
    }
}
