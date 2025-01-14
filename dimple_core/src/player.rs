pub mod track_downloader;

use std::{sync::{mpsc::{Receiver, Sender}, Arc, Mutex, RwLock}, time::{Duration, Instant}};

use threadpool::ThreadPool;
use track_downloader::{TrackDownloadStatus, TrackDownloader};

use crate::{library::Library, model::{Event, Playlist, Track}, notifier::{self, Notifier}};

#[derive(Clone)]
pub struct Player {
    library: Arc<Library>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    downloader: TrackDownloader,
    notifier: Notifier<String>,
    threadpool: ThreadPool,
}

// TODO volume https://github.com/tramhao/termusic/blob/master/playback/src/rusty_backend/sink.rs
// https://github.com/10buttons/awedio
// https://sansan.cat/

impl Player {
    pub fn new(library: Arc<Library>) -> Player {
        let (sender, receiver) = std::sync::mpsc::channel();
        let player = Player {
            library,
            sender,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
            downloader: TrackDownloader::default(),
            notifier: Notifier::new(),
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

    /// Insert into queue before the current song, and skip backwards to the
    /// newly inserted song. The currently playing song, if any, will then
    /// be played after.
    pub fn play_now(&self, track_key: &str) {

    }

    /// Insert into the queue after the current index, so that the song will
    /// be played next.
    pub fn play_next(&self, track_key: &str) {

    }

    /// Append to the end of the queue.
    pub fn play_later(&self, track_key: &str) {

    }

    // pub fn playlist_add(&self, playlist: &Playlist, track_key: &str) {
    //     self.conn().execute("INSERT INTO PlaylistItem 
    //         (key, playlist_key, track_key) 
    //         VALUES (?1, ?2, ?3)",
    //         (&Uuid::new_v4().to_string(), playlist.key.clone().unwrap(), track_key)).unwrap();
    // }

    // pub fn playlist_clear(&self, playlist: &Playlist) {
    //     self.conn().execute("DELETE FROM PlaylistItem
    //         WHERE playlist_key = ?1", (playlist.key.clone().unwrap(),)).unwrap();
    // }    

    pub fn play(&self) {
        self.sender.send(PlayerCommand::Play).unwrap();
    }

    pub fn pause(&self) {
        self.sender.send(PlayerCommand::Pause).unwrap();
    }

    pub fn next(&self) {
        self.scrobble("track_skipped");
        if let Ok(mut shared_state) = self.shared_state.write() {
            shared_state.queue_index = (shared_state.queue_index + 1).min(self.queue().len(&self.library) - 1);
            // TODO change to skip and check if current is playing to use gapless prebuffer
            self.sender.send(PlayerCommand::Stop).unwrap();
        }
    }

    pub fn previous(&self) {
        let mut restarted: bool = false;
        if let Ok(mut shared_state) = self.shared_state.write() {
            const REWIND_SECONDS: u64 = 3;
            if shared_state.track_position.as_secs() >= REWIND_SECONDS {
                restarted = true;
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
        // TODO note this is done outside of the logic above because the above
        // is inside the lock. Should be refactored.
        if restarted {
            self.scrobble("track_restarted");
        }
        else {
            self.scrobble("previous_track");
        }
    }

    pub fn set_queue_index(&self, index: usize) {
        self.shared_state.write().unwrap().queue_index = index;
        self.sender.send(PlayerCommand::Stop).unwrap();
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
        self.queue().tracks(&self.library).get(self.current_queue_index()).cloned()
    }

    pub fn next_queue_track(&self) -> Option<Track> {
        self.queue().tracks(&self.library).get(self.current_queue_index() + 1).cloned()
    }

    pub fn on_change(&self, l: Box<dyn Fn(&String) + Send>) {
        self.notifier.on_notify(l);
    }

    fn emit_change(&self, event: &str) {
        let notifier = self.notifier.clone();
        let event = event.to_string();
        self.threadpool.execute(move || {
            notifier.notify(&event);
        });
    }

    fn player_worker(&self, receiver: Receiver<PlayerCommand>) {
        let inner = playback_rs::Player::new(None).unwrap();
        inner.set_playing(false);
        loop {
            while let Ok(command) = receiver.recv_timeout(Duration::from_millis(100)) {
                match command {
                    PlayerCommand::Play => {
                        inner.set_playing(true);
                    },
                    PlayerCommand::Pause => {
                        inner.set_playing(false);
                    },
                    PlayerCommand::Seek(position) => {
                        inner.seek(position);
                    },
                    PlayerCommand::Skip => {
                        inner.skip();
                    },
                    PlayerCommand::Stop => {
                        inner.stop();
                        self.set_last_loaded_queue_index(None);
                    },
                }
            }

            if !inner.has_next_song() {
                self.load_next_song(&inner);
            }

            let (position, duration) = inner.get_playback_position().unwrap_or_default();
            if let Ok(mut shared_state) = self.shared_state.write() {
                if shared_state.track_position != position {
                    shared_state.track_position = position;
                    self.emit_change("position");
                }
                if shared_state.track_duration != duration {
                    shared_state.track_duration = duration;
                    self.emit_change("duration");
                }
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

    // TODO might be worth making a quick way to backup or dump the data
    // I'm storing in case I kill my database.
    // TODO figure out how to detect a rewind / replay of a section and scrobble
    // them shits.
    fn scrobble(&self, event_type: &str) {
        if let Some(current_track) = self.current_queue_track() {
            // TODO quick hack, getting a feel for this, but also want to be
            // storing the history I'm listening to.
            let timestamp = chrono::Utc::now();
            self.library.save_unlogged(&Event {
                key: None,
                timestamp: timestamp,
                event_type: event_type.to_string(),
                artist: current_track.artist_name(&self.library).clone(),
                album: current_track.album_name(&self.library).clone(),
                title: current_track.title.clone(),
                source_type: "dimple_testing".to_string(),
                source: format!("{}:{}:{:?}:{:?}:{:?}",
                    &timestamp,
                    event_type,
                    &current_track.artist_name(&self.library),
                    &current_track.album_name(&self.library),
                    &current_track.title),
            });
        }
    }

    fn load_next_song(&self, inner: &playback_rs::Player) {
        let last_loaded_queue_index = self.last_loaded_queue_index();
        let current_queue_index = self.current_queue_index();
        match last_loaded_queue_index {
            Some(last_loaded_queue_index) => {
                if last_loaded_queue_index == current_queue_index {
                    self.load_next_available_song(current_queue_index + 1, inner);        
                }
                else {
                    self.scrobble("track_played");
                    self.advance_queue();
                }
            },
            None => {
                self.load_next_available_song(current_queue_index, inner);
            },
        }
    }

    fn load_next_available_song(&self, start_index: usize, inner: &playback_rs::Player) {
        let mut queue_index = start_index;
        // TODO querying the db every 1/10 seconds? Need to copy tracks, or at least
        // the next N tracks into the state and then reload on changes to the playlist.
        let tracks = self.queue().tracks(&self.library);
        loop {
            match tracks.get(queue_index) {
                Some(track) => {
                    match self.downloader.get(&track, &self.library) {
                        TrackDownloadStatus::Downloading => {
                            return
                        },
                        TrackDownloadStatus::Ready(song) => { 
                            inner.play_song_next(&song, None).unwrap();
                            self.set_last_loaded_queue_index(Some(queue_index));
                            return
                        },
                        TrackDownloadStatus::Error(e) => {
                            log::error!("Error loading next track {:?}, trying next. {}", track, e);
                            queue_index += 1;
                            continue
                        },
                    }
                },
                None => {
                    // TODO temp commented out to fix the last song not playing bug
                    // log::info!("End of queue, stopping playback.");
                    // inner.set_playing(false);
                    // inner.stop();
                    return
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

    fn last_loaded_queue_index(&self) -> Option<usize> {
        self.shared_state.read().unwrap()._last_loaded_queue_index
    }

    fn set_last_loaded_queue_index(&self, last_loaded_queue_index: Option<usize>) {
        self.shared_state.write().unwrap()._last_loaded_queue_index = last_loaded_queue_index;
        self.emit_change("last_loaded_queue_index");
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Default)]
struct SharedState {
    queue_index: usize,
    _last_loaded_queue_index: Option<usize>,
    track_duration: Duration,
    track_position: Duration,
    inner_player_state: PlayerState,
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
    use std::{sync::Arc, time::Instant};

    use crate::library::Library;

    use super::Player;

    #[test]
    fn it_works() {
        // Note, if this test is failing randomly make sure it's running in
        // release mode. Symphonia is too slow in debug mode to keep up.
        let _ = env_logger::try_init();
        let library = Arc::new(Library::open_memory());
        let player = Player::new(library.clone());
        library.import("tests/data/media_files");
        let tracks = library.tracks();
        let play_queue = player.queue();
        for track in &tracks[0..3] {
            library.playlist_add(&play_queue, track.key.as_ref().unwrap());
        }
        assert!(!player.is_playing());
        player.play();

        let t = Instant::now();
        while !player.is_playing() && t.elapsed().as_secs() < 3 {}
        assert!(player.is_playing());
        player.pause();
    }
}
