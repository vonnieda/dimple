use std::sync::{Arc};

use rodio::Sink;

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

#[derive(Clone)]
pub struct Player {
    sink: Arc<Sink>,
    librarian: Arc<Librarian>,
    tracks: Vec<Track>,
    current_track_index: usize,
}

impl Player {
    pub fn new(sink: Arc<Sink>, librarian: Arc<Librarian>) -> Self {
        Self {
            sink,
            librarian,
            tracks: Vec::new(),
            current_track_index: 0,
        }
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

