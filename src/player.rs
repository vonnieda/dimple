use std::{sync::{Arc, RwLock}, time::Duration};

use rodio::Sink;

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

#[derive(Clone)]
pub struct Player {
    sink: Arc<Sink>,
    librarian: Arc<Librarian>,
    tracks: Vec<Track>,
    current_track_index: usize,
}

pub type PlayerHandle = Arc<RwLock<Player>>;

// TODO play next track when one finishes
// TODO cache next track
// TODO figure out how to speed up first play
// TODO for gapless, whenever we change tracks we load that track and
//      the next into the sink. And maybe the previous.
//      So then next and previous just do those things on the sound.
        
impl Player {
    pub fn new(sink: Arc<Sink>, librarian: Arc<Librarian>) -> PlayerHandle {
        let myself = Arc::new(RwLock::new(Self {
            sink,
            librarian,
            tracks: Vec::new(),
            current_track_index: 0,
        }));

        let myself_1 = myself.clone();
        std::thread::spawn(move || {
            loop {
                if !myself_1.read().unwrap().sink.empty() {
                    myself_1.read().unwrap().sink.sleep_until_end();
                    log::info!("Playing next track");
                    myself_1.write().unwrap().next();
        }
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        myself
    }

    pub fn add_release(&mut self, release: &Release) {
        for track in &release.tracks {
            self.add_track(track);
        }
    }

    pub fn add_track(&mut self, track: &Track) {
        self.tracks.push(track.clone());
        self.play();
    }

    pub fn play(&mut self) {
        // If the playlist is empty, do nothing.
        if self.tracks.is_empty() {
            return;
        }

        // If the sink is empty, load the current track.
        if self.sink.empty() {
            let track = self.tracks[self.current_track_index].clone();
            self.librarian.stream(&track, &self.sink).unwrap();
        }
        
        // And play it.
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn next(&mut self) {
        // If the playlist is empty, do nothing.
        if self.tracks.is_empty() {
            return;
        }

        // Increment or restart the queue
        self.current_track_index = (self.current_track_index + 1) % self.tracks.len();

        // If we were already playing, stop and play the new track
        if !self.sink.empty() {
            self.sink.clear();
            self.sink.stop();
            // Seems to be a race condition on clearing the sink and playing
            // the next track, so wait to make sure it's done.
            loop {
                if self.sink.empty() {
                    break;
                }
            }
            self.play();
        }
    }

    pub fn previous(&mut self) {
        // If the playlist is empty, do nothing.
        if self.tracks.is_empty() {
            return;
        }

        // Decrement or restart the queue
        if self.current_track_index == 0 {
            self.current_track_index = self.tracks.len() - 1;
        }
        else {
            self.current_track_index -= 1;
        }

        // If we were already playing, stop and play the new track
        if !self.sink.empty() {
            self.sink.clear();
            self.sink.stop();
            // Seems to be a race condition on clearing the sink and playing
            // the next track, so wait to make sure it's done.
            loop {
                if self.sink.empty() {
                    break;
                }
            }
            self.play();
        }
    }

    pub fn current_track(&self) -> Option<Track> {
        if self.tracks.is_empty() {
            return None
        }
        Some(self.tracks[self.current_track_index].clone())
    }

    pub fn next_track(&self) -> Option<Track> {
        None
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }

    pub fn clear(&mut self) {
        self.sink.stop();
        self.tracks.clear();
        self.current_track_index = 0;
    }
}

