use std::{io::Cursor, sync::{mpsc::{self, Receiver, Sender}, Arc, RwLock}, time::Duration};

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

    pub fn stop(&self) {
        self.sender.send(PlayerCommand::Stop).unwrap();
    }

    pub fn next(&self) {
        self.sender.send(PlayerCommand::Next).unwrap();
    }

    pub fn previous(&self) {
        self.sender.send(PlayerCommand::Previous).unwrap();
    }

    pub fn track_duration(&self) -> Duration {
        self.shared_state.read().unwrap().track_duration
    }

    pub fn track_position(&self) -> Duration {
        self.shared_state.read().unwrap().track_position
    }

    pub fn seek(&self, position: Duration) {
        self.sender.send(PlayerCommand::Seek(position)).unwrap();
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

    fn player_worker(&self, receiver: Receiver<PlayerCommand>) {
        let inner = playback_rs::Player::new(None).unwrap();
        inner.set_playing(false);
        loop {
            // Process incoming commands, waiting up to 100ms for one to arrive.
            // This also limits the speed that this loop loops, and the speed
            // at which shared state updates.
            self.process_commands(&receiver, &inner);

            // Load media into the player if needed
            self.load_media(&inner);

            // Update shared state
            // TODO this is failing due to a race condition with the above.
            // I'll need to combine parts of these two functions to get things
            // in order because the song may cease playing between runs of the loop
            // or even during running of the loop.
            self.update_shared_state(&inner);
        }
    }

    /// Process commands on the player thread.
    fn process_commands(&self, receiver: &Receiver<PlayerCommand>, inner: &playback_rs::Player) {
        while let Ok(command) = receiver.recv_timeout(Duration::from_millis(100)) {
            match command {
                PlayerCommand::Play => inner.set_playing(true),
                PlayerCommand::Pause => inner.set_playing(false),
                PlayerCommand::Stop => {
                    self.shared_state.write().unwrap().queue_index = 0;
                    inner.set_playing(false);
                    inner.stop();
                },
                PlayerCommand::Next => {
                    self.queue_next();
                    inner.skip();
                },
                PlayerCommand::Previous => {
                    if let Some((position, _duration)) = inner.get_playback_position() {
                        const REWIND_SECONDS: u64 = 3;
                        if position.as_secs() < REWIND_SECONDS {
                            self.queue_previous();
                            inner.stop();
                        }
                        else {
                            inner.seek(Duration::ZERO);
                        }
                    }
                },
                PlayerCommand::Seek(position) => {
                    inner.seek(position);
                },
            }
        }
    }

    /// Load media, if needed, into the inner player. To support gapless
    /// playback we need to make sure the next track is loaded before the
    /// current one finishes. This ensures that happens, along with loading
    /// the current and/or first track as needed.
    fn load_media(&self, inner: &playback_rs::Player) {
        if !inner.has_current_song() {
            if let Some(track) = self.current_queue_item() {
                log::info!("Loading Track:{:?} {:?}", track.key, track.title);
                let content = self.library.load_track_content(&track).expect("No valid sources found.");
                let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();            
                inner.play_song_now(&song, None).unwrap();
            }
        }
        // TODO this doesn't seem to trigger until we actually set_playing = true
        // which is surprising and messes up the UI by not updating position and
        // duration.

        /// When does the media need to be changed?
        /// - When there is no current song loaded.
        /// - When there is no next song loaded.
        /// - When the player advances automatically from the current to the next.
        /// - When next() is called
        /// - When previous() is called
        /// 
        /// Okay, yea, I think change the commands to not manipulate the player
        /// but to instead manipulate the state, and have the player react
        /// to the changes. I think it fixes everything.
        /// 
        /// next increases the index but doesn't skip or anything
        /// previous decreases """"
        /// state DRIVES the player (thread), rather than the opposite
        /// the only thing we need to detect is auto advance and that just
        /// becomes a "next" command?
        /// 
        /// Note I just realized an issue is that next and previous just should
        /// change the index of the playlist and then the player thread should
        /// react. This will keep the UI snappy and maybe solves the above.

        if !inner.has_next_song() {
            if let Some(track) = self.next_queue_item() {
                log::info!("Loading next Track:{:?} {:?}", track.key, track.title);
                let content = self.library.load_track_content(&track).expect("No valid sources found.");
                let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();            
                inner.play_song_next(&song, None).unwrap();
            }
        }
    }

    /// Update shared state. Distills the state of the inner player into a
    /// single state variable and updates other externally accessible state
    /// that can only be read from this thread.
    fn update_shared_state(&self, inner: &playback_rs::Player) {
        if let Ok(mut shared_state) = self.shared_state.write() {
            shared_state.inner_player_state = match (inner.is_playing(), inner.has_current_song()) {
                (true, true) => PlayerState::Playing,
                (true, false) => PlayerState::Stopped,
                (false, true) => PlayerState::Paused,
                (false, false) => PlayerState::Stopped,
            };

            if let Some((position, duration)) = inner.get_playback_position() {
                shared_state.track_position = position;
                shared_state.track_duration = duration;
            }
            else {
                shared_state.track_position = Duration::ZERO;
                shared_state.track_duration = Duration::ZERO;
            }

            if shared_state.inner_player_was_playing 
                && shared_state.inner_player_had_current
                && shared_state.inner_player_had_next
                && inner.is_playing()
                && inner.has_current_song()
                && (!inner.has_next_song()) {
                log::info!("song finished, and nothing new is loaded, advancing queue");
                shared_state.queue_index = (shared_state.queue_index + 1) % self.queue().len(&self.library);
            }
            shared_state.inner_player_was_playing = inner.is_playing();
            shared_state.inner_player_had_current = inner.has_current_song();
            shared_state.inner_player_had_next = inner.has_next_song();
        }
    }

    fn queue_next(&self) {
        let mut shared_state = self.shared_state.write().unwrap();
        shared_state.queue_index = (shared_state.queue_index + 1) % self.queue().len(&self.library);
    }

    fn queue_previous(&self) {
        let mut shared_state = self.shared_state.write().unwrap();
        shared_state.queue_index = 0.max(shared_state.queue_index - 1);
    }

    pub fn current_queue_index(&self) -> usize {
        self.shared_state.read().unwrap().queue_index
    }

    pub fn current_queue_item(&self) -> Option<Track> {
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

    pub fn next_queue_item(&self) -> Option<Track> {
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

#[derive(Default)]
struct SharedState {
    pub queue_index: usize,
    pub queue_length: usize,
    pub track_duration: Duration,
    pub track_position: Duration,
    pub inner_player_state: PlayerState,
    pub inner_player_was_playing: bool,
    pub inner_player_had_current: bool,
    pub inner_player_had_next: bool,
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
    Next,
    Previous,
    Stop,
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


    // pub fn play(&self) {
    //     let player = playback_rs::Player::new(None).unwrap();
    //     let play_queue = self.play_queue();
    //     for track in play_queue.tracks {            
    //         while player.has_next_song() {
    //             std::thread::sleep(std::time::Duration::from_millis(100));
    //         }
    //         let content = self.library.load_track_content(&track).expect("No valid sources found.");
    //         let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();
    //         player.play_song_next(&song, None).unwrap();
    //     }
    //     while player.has_current_song() {
    //         std::thread::sleep(std::time::Duration::from_millis(100));
    //     }
    // }

