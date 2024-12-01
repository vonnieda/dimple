use std::{sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, time::Duration};

use crate::{library::Library, model::{Model, Playlist}};

#[derive(Clone)]
pub struct Player {
    library: Arc<Library>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    // songs: Arc<RwLock<HashMap<String, MediaStatus>>>,
}

impl Player {
    pub fn new(library: Arc<Library>) -> Player {
        let (sender, receiver) = std::sync::mpsc::channel();
        let player = Player {
            library,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
        };
        {
            let player = player.clone();
            std::thread::spawn(move || player.player_worker(receiver));
        }
        {
            let player = player.clone();
            std::thread::spawn(move || player.download_worker());
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
        // TODO should happen in library.get
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
        self.shared_state.read().unwrap().state.clone()
    }

    fn player_worker(&self, receiver: Receiver<PlayerCommand>) {
        let inner = playback_rs::Player::new(None).unwrap();
        loop {
            // Process incoming commands, waiting up to 100ms for one to arrive.
            // This also limits the speed that this loop loops, and the speed
            // at which shared state updates.
            self.process_commands(&receiver, &inner);

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

    /// Update shared state 
    fn update_shared_state(&self, inner: &playback_rs::Player) {
        self.shared_state.write().unwrap().state = match (inner.is_playing(), inner.has_current_song()) {
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

    fn download_worker(&self) {
    }    
}

#[derive(Default)]
struct SharedState {
    pub index: usize,
    pub duration: Duration,
    pub position: Duration,
    pub state: PlayerState,
}

#[derive(Clone, Debug, Default)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone)]
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
    use std::{sync::Arc, thread::sleep, time::Duration};

    use crate::{library::{self, Library}, scanner::Scanner};

    use super::Player;

    #[test]
    fn it_works() {
        let library = Arc::new(Library::open("file:ee2e5b97-b997-431d-8224-d361e905d071?mode=memory&cache=shared"));
        let player = Player::new(library.clone());
        library.import(&Scanner::scan_directory("tests/data/media_files"));
        let tracks = library.tracks();
        for track in &tracks[0..3] {
            // player.play_queue_add(track.key.as_ref().unwrap());
        }
        player.play();
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

    // fn advance_queue(&self) {
    //     let mut shared_state = self.shared_state.write().unwrap();
    //     shared_state.index = (shared_state.index + 1) % shared_state.queue.len();
    // }

    // pub fn queue(&self) -> Vec<QueuedItem> {
    //     self.shared_state.read().unwrap().queue.clone()
    // }

    // pub fn current_queue_index(&self) -> usize {
    //     self.shared_state.read().unwrap().index
    // }

    // pub fn current_queue_item(&self) -> Option<QueuedItem> {
    //     if let Ok(state) = self.shared_state.read() {
    //         if state.index >= state.queue.len() {
    //             return None;
    //         }
    //         Some(state.queue[state.index].clone())
    //     }
    //     else {
    //         None
    //     }
    // }

    // pub fn next_queue_item(&self) -> Option<QueuedItem> {
    //     if let Ok(state) = self.shared_state.read() {
    //         if state.index + 1 >= state.queue.len() {
    //             return None;
    //         }
    //         Some(state.queue[state.index + 1].clone())
    //     }
    //     else {
    //         None
    //     }
    // }

