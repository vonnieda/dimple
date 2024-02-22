use std::{collections::VecDeque, io::Cursor, sync::{mpsc::{channel, Receiver, Sender}, Arc, RwLock}, thread, time::Duration};

use dimple_core::{collection::Collection, model::{Entities, Recording, RecordingSource}};
use dimple_librarian::librarian::Librarian;
use playback_rs::{Hint, Song};

#[derive(Clone)]
pub struct Player {    
    librarian: Arc<Librarian>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    // play_queue: Arc<RwLock<VecDeque<QueuedItem>>>,
    // play_queue_index: Arc<RwLock<usize>>,
    // index: usize,
    // duration: f32,
    // position: f32,
    // is_playing: bool,
}

#[derive(Default)]
struct SharedState {
    pub queue: Vec<QueuedItem>,
    pub index: usize,
    pub duration: Duration,
    pub position: Duration,
    pub state: PlayerState,
}

impl Player {
    pub fn new(librarian: Arc<Librarian>) -> Player {
        let (sender, receiver) = channel();
        let player = Player {
            librarian,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
        };
        {
            let player = player.clone();
            thread::spawn(move || player.worker(receiver));
        }
        player
    }

    pub fn enqueue(&self, entity: &Entities) {
        match entity {
            Entities::Recording(r) => {
                self.shared_state.write().unwrap().queue.push(QueuedItem {
                    recording: r.clone(),
                    source: None,
                    song: None,
                })
            },
            _ => todo!()
        }
    }

    pub fn queue(&self) -> Vec<QueuedItem> {
        self.shared_state.read().unwrap().queue.clone()
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
        // self.player_state.read().unwrap().duration
        todo!()
    }

    pub fn position(&self) -> Duration {
        // self.player_state.read().unwrap().position
        todo!()
    }

    pub fn seek(&self, position: Duration) {
        self.sender.send(PlayerCommand::Seek(position)).unwrap();
    }

    pub fn state(&self) -> PlayerState {
        self.shared_state.read().unwrap().state.clone()
    }

    /**
     * The playback_rs player is either playing, paused, or stopped, but that
     * is spread across a few properties. This combines them.
     */
    // fn inner_player_state(inner: &playback_rs::Player) -> PlayerState {
    //     match (inner.has_current_song(), inner.is_playing()) {
    //         (true, true) => PlayerState::Playing,
    //         (true, false) => PlayerState::Paused,
    //         (false, true) => panic!("invalid state"), 
    //         (false, false) => PlayerState::Stopped,
    //     }
    // }

    fn worker(&self, receiver: Receiver<PlayerCommand>) {
        // TODO I think that eventually to do all the equalizing and
        // mixing and such I'll need to drop playback_rs and go with the
        // components it's made up of, but for now it works.
        let inner = playback_rs::Player::new(None).unwrap();
        // let mut td = TrackDownloader::new(library);
        // let _current_queue_item:Option<QueueItem> = None;
        // let next_queue_item:Option<QueueItem> = None;
        loop {
            // Process incoming commands, waiting for 100ms for one to arrive.
            // This also limits the speed that this loop loops.
            while let Ok(command) = receiver.recv_timeout(Duration::from_millis(100)) {
                log::info!("a command!");
                match command {
                    PlayerCommand::Play => self.shared_state.write().unwrap().state = PlayerState::Playing,
                    PlayerCommand::Pause => self.shared_state.write().unwrap().state = PlayerState::Paused,
                    PlayerCommand::Stop => self.shared_state.write().unwrap().state = PlayerState::Stopped,
                    PlayerCommand::Next => todo!(),
                    PlayerCommand::Previous => todo!(),
                    PlayerCommand::Seek(position) => {
                        inner.seek(position);
                    },
                }
            }

            match self.state() {
                PlayerState::Stopped => {
                    if inner.is_playing() {
                        inner.stop();
                    }
                },
                PlayerState::Paused => {
                    if inner.is_playing() {
                        inner.set_playing(false);
                    }
                },
                PlayerState::Playing => {
                    if !inner.has_current_song() {
                        if let Some(current_item) = self.current_queue_item() {
                            if let Some(song) = current_item.song {
                                inner.play_song_now(&song, None).unwrap();
                            }
                            else {
                                // TODO Prioritize the download
                                if let Some(stream) = self.librarian.stream(&current_item.recording.entity()) {
                                    let bytes: Vec<_> = stream.collect();
                                    log::info!("downloaded {} bytes", bytes.len());
                                    let song = Song::new(Box::new(Cursor::new(bytes)), 
                                        &Hint::new(), 
                                        None).unwrap();
                                    if let Ok(mut shared_state) = self.shared_state.write() {
                                        let index = shared_state.index;
                                        if let Some(item) = shared_state.queue.get_mut(index) {
                                            item.song = Some(song);
                                        }
                                    }
                                }
                            }
                        }
                        else {
                            // If there is nothing in the queue we can't play.
                            // TODO stuff below fucks this.
                            self.stop();
                        }
                    }

                    if !inner.is_playing() {
                        inner.set_playing(true);
                    }

                    if !inner.has_next_song() {
                        if let Some(next_item) = self.next_queue_item() {
                            if let Some(song) = next_item.song {
                                log::info!(
                                    "Queueing next song with {:?} left in current song...",
                                    inner.get_playback_position());
                                inner.play_song_next(&song, None).unwrap();
                            }
                        }
                    }
                },
            }


            // // TODO this one and the next block can get out of sync with what's
            // // playing  when skipping quickly. Need to keep track of which
            // // track is actually loaded and change it whenever it is wrong.
            // // If the current song is not loaded, load it
            // if !inner.has_current_song() {
            //     if let Some(item) = player_state.read().unwrap().current_queue_item() {
            //         let track = item.track;
            //         match td.get(&track) {
            //             TrackDownloadProgress::Error => todo!(),
            //             TrackDownloadProgress::Downloading => {},
            //             TrackDownloadProgress::Ready(_, song) => inner.play_song_now(&song, None).unwrap(),
            //         }
            //     }
            // }

            // Update shared state
            // if let Some((position, duration)) = inner.get_playback_position() {
            //     self.player_state.write().unwrap().position = position.as_secs_f32();
            //     self.player_state.write().unwrap().duration = duration.as_secs_f32();
            // }
            // else {
            //     self.player_state.write().unwrap().position = 0.0;
            //     self.player_state.write().unwrap().duration = 0.1;
            // }
            // self.player_state.write().unwrap().is_playing = inner.is_playing();

            // Refresh the context
            // TODO this is a hack - UI stuff doesn't belong here, but didn't
            // yet come up with a better way to do it.
            // ctx.request_repaint();
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Debug)]
pub struct QueuedItem {
    pub recording: Recording,
    pub source: Option<RecordingSource>,
    pub song: Option<Song>,
}

#[derive(Clone)]
pub enum PlayerCommand {
    Play,
    Pause,
    Next,
    Previous,
    Stop,
    Seek(Duration),
    // DownloadComplete(Vec<u8>),
}

// TODO will prioritize making sure songs are loaded for the N songs in the
// window around the current one, handling flushing songs when needed.
struct SongLoader {

}

// impl PlayerState {
//     pub fn queue_release(&mut self, release: &Release) {
//         for track in &release.tracks {
//             self.queue_track(release, track);
//         }
//     }

//     pub fn queue_track(&mut self, release: &Release, track: &Track) {
//         self.queue.push(QueueItem {
//             release: release.clone(),
//             track: track.clone()
//         });
//     }

//     pub fn current_queue_item(&self) -> Option<QueueItem> {
//         if self.index >= self.queue.len() {
//             return None;
//         }
//         Some(self.queue[self.index].clone())
//     }

//     pub fn next_queue_item(&self) -> Option<QueueItem> {
//         if self.index + 1 >= self.queue.len() {
//             return None;
//         }
//         Some(self.queue[self.index + 1].clone())
//     }

//     pub fn is_empty(&self) -> bool {
//         self.queue.is_empty()
//     }

//     pub fn next(&mut self) {
//         // Increment or restart the queue
//         self.index = (self.index + 1) % self.queue.len();
//     }

//     pub fn previous(&mut self) {
//         // Decrement or restart the queue
//         if self.index == 0 {
//             self.index = self.queue.len() - 1;
//         }
//         else {
//             self.index -= 1;
//         }
//     }
// }

// pub enum TrackDownloadProgress {
//     Error,
//     Downloading,
//     Ready(Track, Song)
// }

// TODO pre-load other songs in the queue
// struct TrackDownloader {
//     downloads: Arc<RwLock<HashSet<Track>>>,
//     songs: Arc<RwLock<HashMap<Track, Song>>>,
//     library: LibraryHandle,
// }

// impl TrackDownloader {
//     pub fn new(library: LibraryHandle) -> Self {
//         Self {
//             downloads: Arc::new(RwLock::new(HashSet::new())),
//             songs: Arc::new(RwLock::new(HashMap::new())),
//             library,
//         }
//     }

//     pub fn get(&mut self, track: &Track) -> TrackDownloadProgress {
//         if let Some(song) = self.songs.read().unwrap().get(track) {
//             TrackDownloadProgress::Ready(track.clone(), song.clone())
//         }
//         else if self.downloads.read().unwrap().contains(track) {
//             TrackDownloadProgress::Downloading
//         }
//         else {
//             log::info!("downloading {}", track.title);
//             self.downloads.write().unwrap().insert(track.clone());
//             let library = self.library.clone();
//             let track = track.clone();
//             let songs = self.songs.clone();
//             std::thread::spawn(move || {
//                 let stream = library.stream(&track).unwrap();
//                 log::info!("downloaded {} bytes", stream.len());
//                 let song = Song::new(Box::new(Cursor::new(stream)), 
//                     &Hint::new(), 
//                     None).unwrap();
//                 log::info!("converted to song");
//                 songs.write().unwrap().insert(track.clone(), song);
//             });    
//             // TODO remove the download
//             TrackDownloadProgress::Downloading
//         }
//     }
// }