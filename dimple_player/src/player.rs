use std::{collections::{BTreeMap, HashMap, VecDeque}, hash::Hash, io::{Cursor, Error}, ops::Add, sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex, RwLock}, thread, time::Duration};

use dimple_librarian::librarian::Librarian;
use playback_rs::{Hint, Song};

#[derive(Clone)]
pub struct Player {    
    // TODO remove the Arc, Librarian is now safe to clone
    librarian: Arc<Librarian>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    songs: Arc<RwLock<HashMap<String, MediaStatus>>>,
}

#[derive(Clone)]
enum MediaStatus {
    Queued,
    Downloading,
    Ready(Song),
    Error(PlayerError),
}

#[derive(Clone, Debug)]
pub struct QueuedItem {
    // pub entity: Entities,
    // pub source: Option<RecordingSource>,
}

#[derive(Default)]
struct SharedState {
    pub queue: Vec<QueuedItem>,
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
    // DownloadComplete(Vec<u8>),
}

#[derive(Clone)]
enum PlayerError {
    NoSources,
    DownloadFailed,
    UnsupportedFormat,
}

impl Player {
    pub fn new(librarian: Arc<Librarian>) -> Player {
        let (sender, receiver) = channel();
        let player = Player {
            librarian,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
            songs: Default::default(),
        };
        {
            let player = player.clone();
            thread::spawn(move || player.player_worker(receiver));
        }
        {
            let player = player.clone();
            thread::spawn(move || player.download_worker());
        }
        player
    }

    // pub fn enqueue(&self, entity: &Entities) {
    //     match entity {
    //         Entities::Recording(r) => {
    //             self.shared_state.write().unwrap().queue.push(QueuedItem {
    //                 entity: entity.clone(),
    //                 source: None,
    //             })
    //         },
    //         Entities::RecordingSource(r) => {
    //             self.shared_state.write().unwrap().queue.push(QueuedItem {
    //                 entity: entity.clone(),
    //                 source: Some(r.clone()),
    //             })
    //         },
    //         _ => todo!()
    //     }
    // }

    pub fn queue(&self) -> Vec<QueuedItem> {
        self.shared_state.read().unwrap().queue.clone()
    }

    pub fn current_queue_index(&self) -> usize {
        self.shared_state.read().unwrap().index
    }

    pub fn current_queue_item(&self) -> Option<QueuedItem> {
        if let Ok(state) = self.shared_state.read() {
            if state.index >= state.queue.len() {
                return None;
            }
            Some(state.queue[state.index].clone())
        }
        else {
            None
        }
    }

    pub fn next_queue_item(&self) -> Option<QueuedItem> {
        if let Ok(state) = self.shared_state.read() {
            if state.index + 1 >= state.queue.len() {
                return None;
            }
            Some(state.queue[state.index + 1].clone())
        }
        else {
            None
        }
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

    fn advance_queue(&self) {
        let mut shared_state = self.shared_state.write().unwrap();
        shared_state.index = (shared_state.index + 1) % shared_state.queue.len();
    }

    fn player_worker(&self, receiver: Receiver<PlayerCommand>) {
        let inner = playback_rs::Player::new(None).unwrap();
        let mut preloaded = false;
        loop {
            // Process incoming commands, waiting up to 100ms for one to arrive.
            // This also limits the speed that this loop loops, and the speed
            // at which shared state updates.
            // All the state checking below is complex. I think I should try
            // to either boil all the state down to "a" state, or get rid of
            // the reactive stuff and do more in each of these command handlers.
            // And probably add a "Download complete" message.
            while let Ok(command) = receiver.recv_timeout(Duration::from_millis(100)) {
                match command {
                    PlayerCommand::Play => inner.set_playing(true),
                    PlayerCommand::Pause => inner.set_playing(false),
                    PlayerCommand::Stop => {
                        inner.stop();
                        preloaded = false;
                    },
                    PlayerCommand::Next => todo!(),
                    PlayerCommand::Previous => todo!(),
                    PlayerCommand::Seek(position) => {
                        inner.seek(position);
                    },
                }
            }

            // TODO might be better to lock the state for the duration here
            // if inner.is_playing() {
            //     if !inner.has_current_song() {
            //         if let Some(current_item) = self.current_queue_item() {
            //             if let Ok(Some(song)) = self.resolve_song(&current_item.entity) {
            //                 // log::info!("Now playing {}", current_item.entity.name().unwrap());
            //                 inner.play_song_now(&song, None).unwrap();
            //             }
            //         }
            //     }

            //     if inner.has_current_song() && !inner.has_next_song() {
            //         // TODO woops, tired,  but this runs repeatedly while the
            //         // song is downloading. Boo. This is all poop, but it's nearly
            //         // finished poop.
            //         if preloaded {
            //             self.advance_queue();
            //             // log::info!("Now playing {}", self.current_queue_item().unwrap().entity.name().unwrap());
            //         }

            //         if let Some(next_item) = self.next_queue_item() {
            //             if let Ok(Some(song)) = self.resolve_song(&next_item.entity) {
            //                 // log::info!("Next up {}", next_item.entity.name().unwrap());
            //                 inner.play_song_next(&song, None).unwrap();
            //             }
            //         }
            //     }

            //     if inner.has_current_song() && inner.has_next_song() {
            //         preloaded = true;
            //     }
            // }

            // Update shared state
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
    }

    // If I do this right I should be able to launch 2 or 3 of these.
    fn download_worker(&self) {
        // loop {
        //     // Grab a copy of the queue and sort it by distance from the
        //     // current item. This will fill in the downloads "middle out".
        //     // This helps ensure that if the user decides to go to the next
        //     // or previous song we'll likely already have it ready.
        //     let index = self.current_queue_index();
        //     let play_queue: Vec<_> = self.queue();
        //     let mut enumerated: Vec<_> = play_queue.iter().enumerate().collect();
        //     enumerated.sort_by_key(|(i, _item)| index.abs_diff(*i));
        //     for (_i, item) in enumerated.iter().take(6) {
        //         let entity = item.entity.clone();
        //         let key = entity.key().unwrap();
        //         // TODO this needs to be reworked to use downloading, so there is
        //         // and the write lock needs to be maintained until we start
        //         // downloading or another thread could pick up the queued item.
        //         // So, get a write lock, check or insert downloading, start the download, release the write lock, when download is complete swap the value to done or error
        //         let status = self.songs.write().unwrap().entry(key.clone()).or_insert(MediaStatus::Queued).clone();
        //         match status {
        //             MediaStatus::Downloading => {},
        //             MediaStatus::Ready(_) => {},
        //             MediaStatus::Queued => {
        //                 match self.download(&entity) {
        //                     Ok(song) => {
        //                         self.songs.write().unwrap().insert(key, MediaStatus::Ready(song));
        //                     },
        //                     Err(e) => {
        //                         self.songs.write().unwrap().insert(key, MediaStatus::Error(e));
        //                     },
        //                 }
        //             },
        //             MediaStatus::Error(_) => {},
        //         }
        //     }
        //     thread::sleep(Duration::from_millis(100));
        // }
    }

    // fn resolve_song(&self, entity: &Entities) -> Result<Option<Song>, PlayerError> {
    //     match self.songs.read().unwrap().get(&entity.key().unwrap()) {
    //         Some(MediaStatus::Ready(song)) => Ok(Some(song.clone())),
    //         Some(MediaStatus::Downloading) => Ok(None),
    //         Some(MediaStatus::Queued) => Ok(None),
    //         Some(MediaStatus::Error(e)) => Err(e.clone()),
    //         None => Ok(None),
    //     }
    // }

    // fn download(&self, entity: &Entities) -> Result<Song, PlayerError> {
    //     // log::debug!("Downloading {}", entity.name().unwrap());
    //     if let Some(stream) = self.librarian.stream(entity) {
    //         let bytes: Vec<_> = stream.collect();
    //         // log::debug!("Downloaded {} bytes for {}", bytes.len(), entity.name().unwrap());
    //         let song = Song::new(Box::new(Cursor::new(bytes)), 
    //             &Hint::new(), 
    //             None).unwrap();
    //         // log::debug!("Converted {} to song", entity.name().unwrap());
    //         Ok(song)
    //     }
    //     else {
    //         Err(PlayerError::NoSources)
    //     }
    // }
}
