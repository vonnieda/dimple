pub mod track_downloader;

use std::{sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, time::{Duration, Instant}};

use track_downloader::{TrackDownloadStatus, TrackDownloader};

use crate::{library::Library, model::{Artist, Event, LibraryModel, ModelBasics as _, Playlist, Release, Track}, notifier::Notifier};

pub use playback_rs::Song;

// TODO STOPSHIP okay heading to bed. I really thought I had it below, but I didn't. Doesn't work for auto-next.
// I think time to refactor this fuck to Rodio.

#[derive(Clone)]
pub struct Player {
    library: Arc<Library>,
    sender: Sender<PlayerCommand>,
    shared_state: Arc<RwLock<SharedState>>,
    downloader: TrackDownloader,
    pub notifier: Notifier<PlayerEvent>,
}

#[derive(Clone, Debug)]
pub enum PlayerEvent {
    State(PlayerState),
    // TODO I think I want this to be something like SongStartedPlaying(Song)
    CurrentSong(Song),
    Position(Duration),
    Duration(Duration),
    QueueIndex(usize),
}

pub enum PlayWhen {
    Now,
    Next,
    Last,
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

    /// Insert into the queue after the current item, and then skip forward
    /// to the newly added item. If the item currently playing is a Release,
    /// for instance, then the rest of the release will be skipped, not just
    /// the current track.
    pub fn play_now(&self, model: &impl LibraryModel) {
        let index = self.current_queue_index() + 1;
        self.queue().insert(&self.library, model, index);
        self.set_queue_index(index);
        self.play();
    }

    /// Insert into the queue after the current item.
    pub fn play_next(&self, model: &impl LibraryModel) {
        self.queue().insert(&self.library, model, self.current_queue_index() + 1);
        self.play();
    }

    /// Append to the end of the queue.
    pub fn play_later(&self, model: &impl LibraryModel) {
        self.queue().append(&self.library, model);
        self.play();
    }

    pub fn enqueue(&self, key: &str, when: PlayWhen) {
        if let Some(model) = Artist::get(&self.library, key) {
            self.enqueue_helper(&model, when);
        }
        else if let Some(model) = Release::get(&self.library, key) {
            self.enqueue_helper(&model, when);
        }
        else if let Some(model) = Track::get(&self.library, key) {
            self.enqueue_helper(&model, when);
        }
    }

    pub fn play(&self) {
        self.sender.send(PlayerCommand::Play).unwrap();
    }

    pub fn pause(&self) {
        self.sender.send(PlayerCommand::Pause).unwrap();
    }

    pub fn next(&self) {
        self.scrobble("track_skipped");
        self.set_current_queue_index((self.current_queue_index() + 1).min(self.queue().len(&self.library) - 1));
        self.sender.send(PlayerCommand::Stop).unwrap();
    }

    pub fn previous(&self) {
        const REWIND_SECONDS: u64 = 3;
        if self.shared_state.read().unwrap().track_position.as_secs() >= REWIND_SECONDS {
            self.scrobble("track_restarted");
            self.sender.send(PlayerCommand::Seek(Duration::ZERO)).unwrap();
        }
        else {
            let queue_index = self.current_queue_index();
            if queue_index > 0 {
                self.set_current_queue_index(queue_index - 1);
                self.sender.send(PlayerCommand::Stop).unwrap();
            }
            self.scrobble("previous_track");
        }
    }

    pub fn set_queue_index(&self, index: usize) {
        self.set_current_queue_index(index);
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
            None => self.library.insert(&Playlist {
                key: Some(key.to_string()),
                ..Default::default()
            })
        };
        playlist
    }

    pub fn current_queue_index(&self) -> usize {
        self.shared_state.read().unwrap()._queue_index
    }

    pub fn current_queue_track(&self) -> Option<Track> {
        self.queue().tracks(&self.library).get(self.current_queue_index()).cloned()
    }

    pub fn next_queue_track(&self) -> Option<Track> {
        self.queue().tracks(&self.library).get(self.current_queue_index() + 1).cloned()
    }

    fn enqueue_helper(&self, model: &impl LibraryModel, when: PlayWhen) {
        match when {
            PlayWhen::Now => self.play_now(model),
            PlayWhen::Next => self.play_next(model),
            PlayWhen::Last => self.play_later(model),
        };
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

            // // We need to detect when the next song becomes the current song.
            // if had_current_song && had_next_song && inner.has_current_song() && !inner.has_next_song() {
            //     self.set_current_song(self.next_song());
            //     self.set_next_song(None);
            // }
            // had_current_song = self.current_song().is_some();
            // had_next_song = self.next_song().is_some();
            if self.next_song().is_some() && !inner.has_next_song() {
                self.set_current_song(self.next_song());
                self.set_next_song(None);
            }
    
            if !inner.has_current_song() || !inner.has_next_song() {
                self.load_next_song(&inner);
            }

            let (position, duration) = inner.get_playback_position().unwrap_or_default();
            if let Ok(mut shared_state) = self.shared_state.write() {
                if shared_state.track_position != position {
                    shared_state.track_position = position;
                    self.notifier.notify(PlayerEvent::Position(position));
                }
                if shared_state.track_duration != duration {
                    shared_state.track_duration = duration;
                    self.notifier.notify(PlayerEvent::Duration(duration));
                }
                let new_state = match (inner.is_playing(), inner.has_current_song()) {
                    (true, true) => PlayerState::Playing,                    
                    (false, true) => PlayerState::Paused,
                    (true, false) => PlayerState::Stopped,
                    (false, false) => PlayerState::Stopped,
                };
                if shared_state.inner_player_state != new_state {
                    shared_state.inner_player_state = new_state.clone();
                    self.notifier.notify(PlayerEvent::State(new_state));
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
            self.library.save(&Event {
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
        // TODO cpu performance querying the db every 1/10 seconds? Need to
        // copy tracks, or at least the next N tracks into the state and then
        // reload on changes to the playlist.
        let tracks = self.queue().tracks(&self.library);
        loop {
            match tracks.get(queue_index) {
                Some(track) => {
                    match self.downloader.get(&track, &self.library) {
                        TrackDownloadStatus::Downloading => {
                        },
                        TrackDownloadStatus::Ready(song) => { 
                            if !inner.has_current_song() {
                                self.set_current_song(Some(song.clone()));
                                self.set_next_song(None);
                                inner.play_song_now(&song, None).unwrap();
                            }
                            else {
                                inner.play_song_next(&song, None).unwrap();
                                self.set_next_song(Some(song.clone()));
                            }

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
        let queue_len = self.queue().tracks(&self.library).len();
        self.set_current_queue_index((self.current_queue_index() + 1).min(queue_len));
    }

    fn last_loaded_queue_index(&self) -> Option<usize> {
        self.shared_state.read().unwrap()._last_loaded_queue_index
    }

    fn set_last_loaded_queue_index(&self, last_loaded_queue_index: Option<usize>) {
        self.shared_state.write().unwrap()._last_loaded_queue_index = last_loaded_queue_index;
    }

    fn set_current_queue_index(&self, index: usize) {
        self.shared_state.write().unwrap()._queue_index = index;
        self.notifier.notify(PlayerEvent::QueueIndex(index));
    }

    fn current_song(&self) -> Option<Song> {
        self.shared_state.read().unwrap().current_song1.clone()
    }

    fn next_song(&self) -> Option<Song> {
        self.shared_state.read().unwrap().current_song1.clone()
    }

    fn set_current_song(&self, song: Option<Song>) {
        self.shared_state.write().unwrap().current_song1 = song.clone();
        if let Some(song) = song {
            self.notifier.notify(PlayerEvent::CurrentSong(song));
        }
    }

    fn set_next_song(&self, song: Option<Song>) {
        self.shared_state.write().unwrap().next_song1 = song;
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
    _queue_index: usize,
    _last_loaded_queue_index: Option<usize>,
    track_duration: Duration,
    track_position: Duration,
    inner_player_state: PlayerState,
    current_song1: Option<Song>,
    next_song1: Option<Song>,
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

    use crate::{library::Library, model::{ModelBasics as _, Track}};

    use super::Player;

    #[test]
    fn it_works() {
        // Note, if this test is failing randomly make sure it's running in
        // release mode. Symphonia is too slow in debug mode to keep up.
        let _ = env_logger::try_init();
        let library = Arc::new(Library::open_memory());
        let player = Player::new(library.clone());
        library.import("tests/data/media_files");
        let tracks = Track::list(&library);
        for track in &tracks[0..3] {
            player.play_later(track);
        }
        assert!(!player.is_playing());
        player.play();

        let t = Instant::now();
        while !player.is_playing() && t.elapsed().as_secs() < 3 {}
        assert!(player.is_playing());
        player.pause();
    }
}
