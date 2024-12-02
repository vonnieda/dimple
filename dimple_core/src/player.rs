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
        // TODO should happen in library.get or maybe just let the caller
        // do if it they want. Reading potentially thousands of items into
        // a list might not be the goal.
        // In fact, hydrate is just a hard coded single purpose query, and
        // I should probably just drop it.
        // and maybe Playlist should just have a get_track_by_index that can
        // by dynamic if needed.
        playlist.hydrate(&self.library);
        playlist
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

    pub fn duration(&self) -> Duration {
        self.shared_state.read().unwrap().duration
    }

    pub fn position(&self) -> Duration {
        self.shared_state.read().unwrap().position
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
                    inner.stop();
                },
                PlayerCommand::Next => todo!(),
                PlayerCommand::Previous => todo!(),
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
        log::info!("{} {}", inner.has_current_song(), inner.has_next_song());
        if !inner.has_current_song() {
            if let Some(track) = self.current_queue_item() {
                log::info!("Loading Track:{:?} {:?}", track.key, track.title);
                let content = self.library.load_track_content(&track).expect("No valid sources found.");
                log::info!("track content.len = {}", content.len());
                let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();            
                log::info!("song loaded");
                inner.play_song_now(&song, None).unwrap();
            }
        }
        if !inner.has_next_song() {
            if let Some(track) = self.next_queue_item() {
                log::info!("Loading next Track:{:?} {:?}", track.key, track.title);
                let content = self.library.load_track_content(&track).expect("No valid sources found.");
                log::info!("next track content.len = {}", content.len());
                let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None).unwrap();            
                log::info!("next song loaded");
                inner.play_song_next(&song, None).unwrap();
            }
        }
    }

    /// Update shared state. Distills the state of the inner player into a
    /// single state variable and updates other externally accessible state
    /// that can only be read from this thread.
    fn update_shared_state(&self, inner: &playback_rs::Player) {
        self.shared_state.write().unwrap().inner_player_state = match (inner.is_playing(), inner.has_current_song()) {
            (true, true) => PlayerState::Playing,
            (true, false) => PlayerState::Stopped,
            (false, true) => PlayerState::Paused,
            (false, false) => PlayerState::Stopped,
        };
        if let Some((position, duration)) = inner.get_playback_position() {
            self.shared_state.write().unwrap().position = position;
            self.shared_state.write().unwrap().duration = duration;
        }
        else {
            self.shared_state.write().unwrap().position = Duration::ZERO;
            self.shared_state.write().unwrap().duration = Duration::ZERO;
        }
    }

    fn advance_queue(&self) {
        let mut shared_state = self.shared_state.write().unwrap();
        shared_state.index = (shared_state.index + 1) % self.queue().len();
    }

    /// TODO note this is a remnant of a refactor and can be factored out
    /// eventually once the API settles.
    pub fn queue(&self) -> Vec<Track> {
        self.play_queue().tracks
    }

    pub fn current_queue_index(&self) -> usize {
        self.shared_state.read().unwrap().index
    }

    pub fn current_queue_item(&self) -> Option<Track> {
        if let Ok(state) = self.shared_state.read() {
            if state.index >= self.queue().len() {
                return None;
            }
            Some(self.queue()[state.index].clone())
        }
        else {
            None
        }
    }

    pub fn next_queue_item(&self) -> Option<Track> {
        if let Ok(state) = self.shared_state.read() {
            if state.index + 1 >= self.queue().len() {
                return None;
            }
            Some(self.queue()[state.index + 1].clone())
        }
        else {
            None
        }
    }
}

#[derive(Default)]
struct SharedState {
    pub index: usize,
    pub duration: Duration,
    pub position: Duration,
    pub inner_player_state: PlayerState,
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
        let play_queue = player.play_queue();

        std::thread::sleep(Duration::from_secs(5));
        dbg!("loading tracks");
        for track in &tracks[0..3] {
            library.playlist_add(&play_queue, track.key.as_ref().unwrap());
        }
        dbg!("loaded");

        std::thread::sleep(Duration::from_secs(5));
        dbg!("playing");
        player.play();
        dbg!("played");

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

