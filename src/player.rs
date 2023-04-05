use std::sync::{Arc};

use rodio::Sink;

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

#[derive(Clone)]
pub struct Player {
    sink: Arc<Sink>,
    librarian: Arc<Librarian>,
    tracks: Vec<Track>,
    current_track_index: Option<usize>,
}

impl Player {
    pub fn new(sink: Arc<Sink>, librarian: Arc<Librarian>) -> Self {
        Self {
            sink,
            librarian,
            tracks: Vec::new(),
            current_track_index: None,
        }
    }

    pub fn clear(&mut self) {
        self.sink.stop();
        self.tracks.clear();
        self.current_track_index = None;
    }

    pub fn play(&mut self) {
        // If the playlist is empty, return.
        if self.tracks.is_empty() {
            return;
        }
        // If there is no "current" track, set it to the first track in the list.
        if self.current_track_index.is_none() {
            self.current_track_index = Some(0);
        }

        // If the sink is not playing anything, load the current track.
        if self.sink.len() == 0 {
            let track = &self.tracks[self.current_track_index.unwrap()];
            self.librarian.stream(track, &self.sink).unwrap();
        }

        // And play it.
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn next(&mut self) {
        // Stop any current playback.
        self.sink.clear();

        // If there's nothing in the queue we're done.
        if self.tracks.is_empty() {
            return;
        }

        // Increment or restart the play queue.
        self.current_track_index = self.current_track_index.map(|index| (index + 1) % self.tracks.len());

        let track = &self.tracks[self.current_track_index.unwrap()];
        self.librarian.stream(track, &self.sink).unwrap();

        self.sink.play();
    }

    pub fn current_track(&self) -> Option<Track> {
        self.current_track_index.map(|index| self.tracks[index].clone())
    }

    pub fn next_track(&self) -> Option<Track> {
        None
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
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
}

