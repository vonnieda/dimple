pub mod track_downloader;

use std::{sync::{mpsc::{Receiver, Sender}, Arc, Mutex, RwLock}, time::Duration};

use playback_rs::Song;
use threadpool::ThreadPool;
use track_downloader::{TrackDownloadStatus, TrackDownloader};

use crate::{library::Library, model::{Playlist, Track}};

type ChangeListener = Arc<Box<dyn Fn(&Player, &str) + Send + std::marker::Sync + 'static>>;

#[derive(Clone)]
pub struct Player {
    library: Arc<Library>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    downloader: TrackDownloader,
    change_listeners: Arc<Mutex<Vec<ChangeListener>>>,
    threadpool: ThreadPool,
}

impl Player {
    pub fn new(library: Arc<Library>) -> Player {
        let (sender, receiver) = std::sync::mpsc::channel();
        let player = Player {
            library,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
            downloader: TrackDownloader::default(),
            change_listeners: Arc::new(Mutex::new(vec![])),
            threadpool: ThreadPool::new(1),
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
            // TODO change to skip and check if current is playing to use gapless prebuffer
            self.sender.send(PlayerCommand::Stop).unwrap();
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
                    self.sender.send(PlayerCommand::Stop).unwrap();
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

        self.threadpool.execute(move || {
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
                    PlayerCommand::Seek(position) => {
                        inner.seek(position);
                        self.emit_change("position");
                    },
                    PlayerCommand::Skip => inner.skip(),
                    PlayerCommand::Stop => {
                        inner.stop();
                        self.shared_state.write().unwrap().last_loaded_queue_index = None;
                    },
                }
            }

            if !inner.has_next_song() {
                self.load_next_song(&inner);
            }

            let (position, duration) = inner.get_playback_position().unwrap_or_default();
            if let Ok(mut shared_state) = self.shared_state.write() {
                shared_state.track_position = position;
                shared_state.track_duration = duration;
                let new_state = match (inner.is_playing(), inner.has_current_song()) {
                    (true, true) => PlayerState::Playing,                    
                    (false, true) => PlayerState::Paused,
                    (true, false) => PlayerState::Stopped,
                    (false, false) => PlayerState::Stopped,
                };
                if shared_state.inner_player_state != new_state {
                    shared_state.inner_player_state = new_state.clone();
                    self.emit_change("state");
                }
            }
        }
    }

    fn load_next_song(&self, inner: &playback_rs::Player) {
        let last_loaded_queue_index = self.shared_state.read().unwrap().last_loaded_queue_index;
        let current_queue_index = self.current_queue_index();
        match last_loaded_queue_index {
            Some(last_loaded_queue_index) => {
                if last_loaded_queue_index == current_queue_index {
                    // log::info!("current song was loaded, preloading next song for gapless playback");
                    self.load_next_available_song(current_queue_index + 1, inner);        
                }
                else {
                    // log::info!("the previously loaded song advanced, advancing the queue");
                    self.advance_queue();
                }
            },
            None => {
                // log::info!("no song has been loaded, loading the current song");
                self.load_next_available_song(current_queue_index, inner);
            },
        }
    }

    fn load_next_available_song(&self, start_index: usize, inner: &playback_rs::Player) -> bool {
        let mut queue_index = start_index;
        loop {
            // TODO querying the db every 1/10 seconds?
            match self.queue().tracks(&self.library).get(queue_index) {
                Some(track) => {
                    match self.downloader.get(&track, &self.library) {
                        TrackDownloadStatus::Downloading => {
                            return false
                        },
                        TrackDownloadStatus::Ready(song) => { 
                            inner.play_song_next(&song, None).unwrap();
                            self.shared_state.write().unwrap().last_loaded_queue_index = Some(queue_index);
                            return true
                        },
                        TrackDownloadStatus::Error(e) => {
                            log::error!("Error loading next track {:?}, trying next. {}", track, e);
                            queue_index += 1;
                            continue
                        },
                    }
                },
                None => {
                    log::warn!("Reached end of queue looking for next song. Giving up.");
                    return false
                },
            }
        }
    }

    /// Advances the queue, stopping at queue_len
    fn advance_queue(&self) {
        let queue_index = self.shared_state.read().unwrap().queue_index;
        let queue_len = self.queue().tracks(&self.library).len();
        self.shared_state.write().unwrap().queue_index = (queue_index + 1).min(queue_len);
    }
}

#[derive(Default)]
struct SharedState {
    queue_index: usize,
    last_loaded_queue_index: Option<usize>,
    track_duration: Duration,
    track_position: Duration,
    inner_player_state: PlayerState,
}

impl SharedState {
    pub fn set_queue_index(&mut self, queue_index: usize, player: &Player) {
        self.queue_index = queue_index;
        player.emit_change("queue_index");
    }
    
    pub fn set_track_duration(&mut self, track_duration: &Duration, player: &Player) {
        self.track_duration = track_duration.clone();
        player.emit_change("track_duration");
    }
    
    pub fn set_track_position(&mut self, track_position: &Duration, player: &Player) {
        self.track_position = track_position.clone();
        player.emit_change("track_position");
    }
    
    pub fn set_inner_player_state(&mut self, inner_player_state: &PlayerState, player: &Player) {
        self.inner_player_state = inner_player_state.clone();
        player.emit_change("inner_player_state");
    }
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
    Skip,
    Stop,
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
