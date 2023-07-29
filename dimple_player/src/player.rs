// TODO play next track when one finishes
// TODO figure out how to speed up first play. I think this is because
//      the method I'm using to get the track downloads the whole thing
//      first instead of streaming from the start.
// TODO I had to break this up into weird pieces to make things threaded,
// so I need to revisit that and either document it, or clean it up.

use std::{sync::{Arc, RwLock, mpsc::{Sender, Receiver}}, fmt::Debug, io::Cursor, collections::{HashMap, HashSet}, time::Duration};

use dimple_core::{model::{Release, Track}, library::LibraryHandle};
use playback_rs::{Song, Hint};

pub struct Player {    
    sender: Sender<PlayerCommand>,
    player_state: Arc<RwLock<PlayerState>>,
}

impl Player {
    pub fn new(library: LibraryHandle) -> PlayerHandle {
        let (sender, receiver) = std::sync::mpsc::channel::<PlayerCommand>();

        let play_queue = Arc::new(RwLock::new(PlayerState::default()));

        let play_queue_1 = play_queue.clone();
        std::thread::spawn(move || Self::run(receiver, library, 
            play_queue_1));

        Arc::new(RwLock::new(Self {
            sender,
            player_state: play_queue,
        }))
    }

    pub fn queue(&self) -> Vec<QueueItem> {
        self.player_state.read().unwrap().queue.clone()
    }

    pub fn queue_release(&mut self, release: &Release) {
        self.player_state.write().unwrap().queue_release(release);
    }

    pub fn queue_track(&mut self, release: &Release, track: &Track) {
        self.player_state.write().unwrap().queue_track(release, track);
    }

    pub fn current_queue_item(&self) -> Option<QueueItem> {
        self.player_state.read().unwrap().current_queue_item()
    }

    pub fn next_queue_item(&self) -> Option<QueueItem> {
        self.player_state.read().unwrap().next_queue_item()
    }

    pub fn play(&mut self) {
        self.sender.send(PlayerCommand::Play).unwrap();
    }

    pub fn pause(&self) {
        self.sender.send(PlayerCommand::Pause).unwrap();
    }

    pub fn next(&mut self) {
        self.sender.send(PlayerCommand::Next).unwrap();
    }

    pub fn previous(&mut self) {
        self.sender.send(PlayerCommand::Previous).unwrap();
    }

    pub fn duration(&self) -> f32 {
        self.player_state.read().unwrap().duration
    }

    pub fn position(&self) -> f32 {
        self.player_state.read().unwrap().position
    }

    pub fn seek(&self, position: f32) {
        self.sender.send(PlayerCommand::Seek(position)).unwrap();
    }

    pub fn is_playing(&self) -> bool {
        self.player_state.read().unwrap().is_playing
    }

    pub fn run(receiver: Receiver<PlayerCommand>, 
        library: LibraryHandle,
        player_state: Arc<RwLock<PlayerState>>) {

        let inner = playback_rs::Player::new(None).unwrap();
        let mut td = TrackDownloader::new(library);
        let _current_queue_item:Option<QueueItem> = None;
        let next_queue_item:Option<QueueItem> = None;
        loop {
            // First, we check if the song has advanced to the next song. The
            // playback-rs library doesn't give us access to the current
            // track information once it's set so we shadow it in
            // current_queue_item and next_queue_item. So, if the player
            // has no next song set, but we think one should have been, then
            // we assume the song has advanced.
            if !inner.has_next_song() && next_queue_item.is_some() {
                // If there is another song in the queue 
                // player_state
            }



            // Process incoming commands
            // TODO process all in the queue.
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(PlayerCommand::Play) => {
                    inner.set_playing(true);
                },
                Ok(PlayerCommand::Next) => {
                    player_state.write().unwrap().next();
                    inner.skip();
                },
                Ok(PlayerCommand::Previous) => {
                    player_state.write().unwrap().previous();
                    inner.stop();
                },
                Ok(PlayerCommand::Seek(position)) => {
                    inner.seek(Duration::from_secs_f32(position));
                },
                Ok(PlayerCommand::Pause) => {
                    inner.set_playing(false);
                },
                Err(_) => {},
            }

            // TODO this one and the next block can get out of sync with what's
            // playing  when skipping quickly. Need to keep track of which
            // track is actually loaded and change it whenever it is wrong.
            // If the current song is not loaded, load it
            if !inner.has_current_song() {
                if let Some(item) = player_state.read().unwrap().current_queue_item() {
                    let track = item.track;
                    match td.get(&track) {
                        TrackDownloadProgress::Error => todo!(),
                        TrackDownloadProgress::Downloading => {},
                        TrackDownloadProgress::Ready(_, song) => inner.play_song_now(&song, None).unwrap(),
                    }
                }
            }

            // TODO not taking care of switching the queue song when the next
            // song plays in.

            // If the next song is not loaded, load it
            if !inner.has_next_song() {
                if let Some(item) = player_state.read().unwrap().next_queue_item() {
                    let track = item.track;
                    match td.get(&track) {
                        TrackDownloadProgress::Error => todo!(),
                        TrackDownloadProgress::Downloading => {},
                        TrackDownloadProgress::Ready(_, song) => inner.play_song_next(&song, None).unwrap(),
                    }
                }
            }

            // Update shared state
            if let Some((position, duration)) = inner.get_playback_position() {
                player_state.write().unwrap().position = position.as_secs_f32();
                player_state.write().unwrap().duration = duration.as_secs_f32();
            }
            else {
                player_state.write().unwrap().position = 0.0;
                player_state.write().unwrap().duration = 0.1;
            }
            player_state.write().unwrap().is_playing = inner.is_playing();

            // Refresh the context
            // TODO this is a hack - UI stuff doesn't belong here, but didn't
            // yet come up with a better way to do it.
            // ctx.request_repaint();
        }
    }
}

#[derive(Clone, Debug)]
pub struct QueueItem {
    pub release: Release,
    pub track: Track,
}

pub type PlayerHandle = Arc<RwLock<Player>>;

#[derive(Clone)]
pub enum PlayerCommand {
    Play,
    Pause,
    Next,
    Previous,
    Seek(f32),
    // DownloadComplete(Vec<u8>),
}

#[derive(Default, Clone, Debug)]
pub struct PlayerState {
    pub queue: Vec<QueueItem>,
    pub index: usize,
    pub duration: f32,
    pub position: f32,
    pub is_playing: bool,
}

impl PlayerState {
    pub fn queue_release(&mut self, release: &Release) {
        for track in &release.tracks {
            self.queue_track(release, track);
        }
    }

    pub fn queue_track(&mut self, release: &Release, track: &Track) {
        self.queue.push(QueueItem {
            release: release.clone(),
            track: track.clone()
        });
    }

    pub fn current_queue_item(&self) -> Option<QueueItem> {
        if self.index >= self.queue.len() {
            return None;
        }
        Some(self.queue[self.index].clone())
    }

    pub fn next_queue_item(&self) -> Option<QueueItem> {
        if self.index + 1 >= self.queue.len() {
            return None;
        }
        Some(self.queue[self.index + 1].clone())
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn next(&mut self) {
        // Increment or restart the queue
        self.index = (self.index + 1) % self.queue.len();
    }

    pub fn previous(&mut self) {
        // Decrement or restart the queue
        if self.index == 0 {
            self.index = self.queue.len() - 1;
        }
        else {
            self.index -= 1;
        }
    }
}

pub enum TrackDownloadProgress {
    Error,
    Downloading,
    Ready(Track, Song)
}

// TODO pre-load other songs in the queue
struct TrackDownloader {
    downloads: Arc<RwLock<HashSet<Track>>>,
    songs: Arc<RwLock<HashMap<Track, Song>>>,
    library: LibraryHandle,
}

impl TrackDownloader {
    pub fn new(library: LibraryHandle) -> Self {
        Self {
            downloads: Arc::new(RwLock::new(HashSet::new())),
            songs: Arc::new(RwLock::new(HashMap::new())),
            library,
        }
    }

    pub fn get(&mut self, track: &Track) -> TrackDownloadProgress {
        if let Some(song) = self.songs.read().unwrap().get(track) {
            TrackDownloadProgress::Ready(track.clone(), song.clone())
        }
        else if self.downloads.read().unwrap().contains(track) {
            TrackDownloadProgress::Downloading
        }
        else {
            log::info!("downloading {}", track.title);
            self.downloads.write().unwrap().insert(track.clone());
            let library = self.library.clone();
            let track = track.clone();
            let songs = self.songs.clone();
            std::thread::spawn(move || {
                let stream = library.stream(&track).unwrap();
                log::info!("downloaded {} bytes", stream.len());
                let song = Song::new(Box::new(Cursor::new(stream)), 
                    &Hint::new(), 
                    None).unwrap();
                log::info!("converted to song");
                songs.write().unwrap().insert(track.clone(), song);
            });    
            // TODO remove the download
            TrackDownloadProgress::Downloading
        }
    }
}