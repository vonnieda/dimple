use std::sync::{Arc, Mutex};

use rodio::Sink;

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

#[derive(Clone)]
pub struct Player {
    sink: Arc<Sink>,
    librarian: Arc<Librarian>,
    playlist: Vec<Track>,
    current_track_index: Option<usize>,
}

impl Player {
    pub fn new(sink: Arc<Sink>, librarian: Arc<Librarian>) -> Self {
        Self {
            sink,
            librarian,
            playlist: Vec::new(),
            current_track_index: None,
        }
    }

    pub fn play(&mut self) {
        // If the playlist is empty, return.
        if self.playlist.len() == 0 {
            return;
        }
        // If there is no "current" track, set it to the first track in the list.
        if self.current_track_index.is_none() {
            self.current_track_index = Some(0);
        }

        // If the sink is not playing anything, load the current track.
        if self.sink.len() == 0 {
            let track = &self.playlist[self.current_track_index.unwrap()];
            self.librarian.stream(track, &self.sink);
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
        if self.playlist.len() == 0 {
            return;
        }

        // Increment or restart the play queue.
        self.current_track_index = self.current_track_index.map_or(None, |index| {
            Some((index + 1) % self.playlist.len())
        });

        let track = &self.playlist[self.current_track_index.unwrap()];
        self.librarian.stream(track, &self.sink);

        self.sink.play();
    }

    pub fn current_track(&self) -> Option<Track> {
        match self.current_track_index {
            Some(index) => Some(self.playlist[index].clone()),
            None => None,
        }
    }

    pub fn next_track(&self) -> Option<Track> {
        None
    }

    pub fn add_release(&mut self, release: &Release) {
        for track in &release.tracks {
            self.add_track(&track);
        }
    }

    pub fn add_track(&mut self, track: &Track) {
        self.playlist.push(track.clone());
        self.play();
    }
}

