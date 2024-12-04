pub mod track_downloader;

use std::{sync::{mpsc::{Receiver, Sender}, Arc, Mutex, RwLock}, time::Duration};

use track_downloader::{TrackDownloadStatus, TrackDownloader};

use crate::{library::Library, model::{Playlist, Track}};

type ChangeListener = Arc<Box<dyn Fn(&Player, &str) + Send + std::marker::Sync + 'static>>;

#[derive(Clone)]
pub struct Player {
    library: Arc<Library>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    track_downloader: TrackDownloader,
    change_listeners: Arc<Mutex<Vec<ChangeListener>>>,
}

impl Player {
    pub fn new(library: Arc<Library>) -> Player {
        let (sender, receiver) = std::sync::mpsc::channel();
        let player = Player {
            library,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
            track_downloader: TrackDownloader::default(),
            change_listeners: Arc::new(Mutex::new(vec![])),
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
            if shared_state.track_position.as_secs() >= REWIND_SECONDS {
                self.sender.send(PlayerCommand::Seek(Duration::ZERO)).unwrap();
            }
            else {
                let queue_index = shared_state.queue_index;
                if queue_index > 0 {
                    shared_state.queue_index = queue_index - 1;
                }
            }
        }
    }

    pub fn set_queue_index(&self, index: usize) {
        self.shared_state.write().unwrap().queue_index = index;
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

    pub fn on_change(&self, callback: impl Fn(&Player, &str) + Send + std::marker::Sync + 'static) {
        let mut listeners = self.change_listeners.lock().unwrap();
        listeners.push(Arc::new(Box::new(callback)));
    }

    fn emit_change(&self, event: &str) {
        let listeners = self.change_listeners.lock().unwrap().clone();
        let player = self.clone();
        let event = event.to_string();
            
        std::thread::spawn(move || {
            for callback in listeners {
                callback(&player, &event);
            }
        });
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

            // TODO pretty sure this is still messing stuff up when using next()
            // okay yea, just tested with this commented out (and next track loading
            // commented out too) and the interactions were all correct and smooth

            // If the player has no next song set, but we had set one previously
            // then the current song finished and the next song is now playing.
            // We need to advance the queue and update the state so that it
            // reflects which song is now loaded.
            // if !inner.has_next_song() && self.shared_state.read().unwrap().next_loaded_track.is_some() {
            //     log::warn!("song finished, advancing the queue!");
            //     if let Ok(mut shared_state) = self.shared_state.write() {
            //         // TODO detect end of queue and send pause or stop
            //         shared_state.current_loaded_track = shared_state.next_loaded_track.clone();
            //         shared_state.next_loaded_track = None;
            //     }
            //     self.next();
            // }

            // Okay, thought about this more and I think I need to think in
            // terms of next song only. I only ever worry about loading
            // a next song when one isn't there. And if one isn't there and
            // I already loaded one, that's a song finished.

            if let Some(current_queue_track) = self.current_queue_track() {
                if let Ok(mut shared_state) = self.shared_state.write() {
                    if shared_state.current_loaded_track != Some(current_queue_track.clone()) {
                        log::warn!("current track mismatch, loading current track");
                        let status = self.track_downloader.get(&current_queue_track, &self.library);
                        if let TrackDownloadStatus::Ready(song) = status {
                            inner.play_song_now(&song, None).unwrap();
                            shared_state.current_loaded_track = Some(current_queue_track.clone());
                        }
                    }
                }
            }

            // // This one has to check if playing because the inner player, when
            // // first loaded with a track but not yet playing, says it has both
            // // a current and a next track even though only one has been loaded. 
            // // This ensures that condition is bypassed so that we don't
            // // accidentally load the "next" track over the freshly loaded
            // // "current" track.
            // if self.state() == PlayerState::Playing {
            //     if let Some(next_queue_track) = self.next_queue_track() {
            //         if let Ok(mut shared_state) = self.shared_state.write() {
            //             if shared_state.next_loaded_track != Some(next_queue_track.clone()) {
            //                 log::warn!("next track mismatch, loading next track");
            //                 let status = self.track_downloader.get(&next_queue_track, &self.library);
            //                 if let TrackDownloadStatus::Ready(song) = status {
            //                     inner.play_song_next(&song, None).unwrap();
            //                     shared_state.next_loaded_track = Some(next_queue_track.clone());
            //                 }
            //             }
            //         }
            //     }
            // }

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
            self.emit_change("");
        }
    }
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
