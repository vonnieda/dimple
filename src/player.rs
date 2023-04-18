// TODO play next track when one finishes
// TODO figure out how to speed up first play. I think this is because
//      the method I'm using to get the track downloads the whole thing
//      first instead of streaming from the start.
// TODO figure out how I'm getting duration and position back

use std::{sync::{Arc, RwLock, mpsc::{Sender, Receiver}}, fmt::Debug, time::{Duration, Instant}, io::Cursor};

use playback_rs::{Song, Hint};

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

pub struct Player {    
    sender: Sender<PlayerCommand>,
    receiver: Receiver<PlayerCommand>,
    player_state: Arc<RwLock<PlayerState>>,
}

impl Player {
    pub fn new(librarian: Arc<Librarian>) -> PlayerHandle {
        let (sender_1, receiver_1) = std::sync::mpsc::channel::<PlayerCommand>();
        let (sender_2, receiver_2) = std::sync::mpsc::channel::<PlayerCommand>();

        let play_queue = Arc::new(RwLock::new(PlayerState::default()));

        let play_queue_1 = play_queue.clone();
        std::thread::spawn(move || Self::run(sender_2, receiver_1, librarian, play_queue_1));

        Arc::new(RwLock::new(Self {
            sender: sender_1,
            receiver: receiver_2,
            player_state: play_queue,
        }))
    }

    pub fn queue_release(&mut self, release: &Release) {
        self.player_state.write().unwrap().queue_release(release);
        self.play();
    }

    pub fn queue_track(&mut self, release: &Release, track: &Track) {
        self.player_state.write().unwrap().queue_track(release, track);
        self.play();
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

    pub fn run(_sender: Sender<PlayerCommand>, 
        receiver: Receiver<PlayerCommand>, 
        librarian: Arc<Librarian>,
        player_state: Arc<RwLock<PlayerState>>) {

        let inner = playback_rs::Player::new(None).unwrap();
        loop {
            // Process any new commands and by way of the timeout, pause
            // this loop for up to 100ms. 
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(PlayerCommand::Play) => {
                    inner.set_playing(true);
                },
                Ok(PlayerCommand::Next) => {
                    inner.skip();
                },
                Ok(PlayerCommand::Previous) => {
                    todo!();
                },
                Ok(PlayerCommand::Seek(position)) => {
                    inner.seek(Duration::from_secs_f32(position));
                },
                Ok(PlayerCommand::Pause) => {
                    inner.set_playing(false);
                },
                Err(_) => {},
            }

            if !inner.has_current_song() {
                if let Some(item) = player_state.read().unwrap().current_queue_item() {
                    let track = item.track;
                    let song = Self::get_song(librarian.clone(), &track);
                    inner.play_song_now(&song, None).unwrap();
                }
            }

            if !inner.has_next_song() {
                if let Some(item) = player_state.read().unwrap().next_queue_item() {
                    let track = item.track;
                    let song = Self::get_song(librarian.clone(), &track);
                    inner.play_song_next(&song, None).unwrap();
                }
            }

            if let Some((position, duration)) = inner.get_playback_position() {
                player_state.write().unwrap().position = position.as_secs_f32();
                player_state.write().unwrap().duration = duration.as_secs_f32();
            }
            else {
                player_state.write().unwrap().position = 0.0;
                player_state.write().unwrap().duration = 0.0;
            }
        }
    }

    fn get_song(librarian: Arc<Librarian>, track: &Track) -> Song {
        let t = Instant::now();
        log::info!("downloading {}", track.title);
        let stream = librarian.stream(track).unwrap();
        log::info!("downloaded {} bytes in {} seconds", 
            stream.len(), 
            Instant::now().duration_since(t).as_secs_f32());
        let song = Song::new(Box::new(Cursor::new(stream)), 
            &Hint::new(), 
            None).unwrap();
        log::info!("converted to song");
        song
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
}

#[derive(Default, Clone, Debug)]
pub struct PlayerState {
    pub queue: Vec<QueueItem>,
    pub index: usize,
    pub duration: f32,
    pub position: f32,
}

// TODO i think this becomes the primary class and the stuff above becomes
// the player inteface or something
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

