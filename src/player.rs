use std::{sync::{Arc, RwLock}, fmt::Debug};

use rodio::Sink;

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

pub struct Player {
    sink: Arc<Sink>,
    librarian: Arc<Librarian>,
    pub queue: Vec<QueueItem>,
    current_queue_item_index: usize,
    // TODO temporary, just so we can play with the slider.
    position: RwLock<f32>,
}

// impl Debug for Arc<Sink> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Arc").finish()
//     }
// }

#[derive(Clone, Debug)]
pub struct QueueItem {
    pub release: Release,
    pub track: Track,
}

pub type PlayerHandle = Arc<RwLock<Player>>;

// TODO play next track when one finishes
// TODO cache next track
// TODO figure out how to speed up first play. I think this is because
//      the method I'm using to get the track downloads the whole thing
//      first instead of streaming from the start.
// TODO for gapless, whenever we change tracks we load that track and
//      the next into the sink. And maybe the previous.
//      So then next and previous just do those things on the sound.

/// The player maintains an editable play queue of release-tracks. release-track
/// because each track has to be associated with a release to get metadata
/// about it.
/// 
/// The player handles fetching, caching, and playing tracks. 
/// 
/// The Librarian needs to be able to make the decision to supply the track from
/// local storage or to try to stream it. 
impl Player {
    pub fn new(sink: Arc<Sink>, librarian: Arc<Librarian>) -> PlayerHandle {
        Arc::new(RwLock::new(Self {
            sink,
            librarian,
            queue: Vec::new(),
            current_queue_item_index: 0,
            position: RwLock::new(0.0),
        }))

        // let player_1 = player.clone();
        // std::thread::spawn(move || {
        //     loop {
        //         if !player_1.read().unwrap().sink.empty() {
        //             player_1.read().unwrap().sink.sleep_until_end();
        //             log::info!("Playing next track");
        //             player_1.write().unwrap().next();
        //         }
        //         std::thread::sleep(Duration::from_millis(100));
        //     }
        // });
    }

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
        self.play();
    }

    pub fn current_queue_item(&self) -> Option<QueueItem> {
        if self.current_queue_item_index >= self.queue.len() {
            return None;
        }
        Some(self.queue[self.current_queue_item_index].clone())
    }

    pub fn next_queue_item(&self) -> Option<QueueItem> {
        if self.current_queue_item_index + 1 >= self.queue.len() {
            return None;
        }
        Some(self.queue[self.current_queue_item_index + 1].clone())
    }

    pub fn play(&mut self) {
        // If the playlist is empty, do nothing.
        if self.queue.is_empty() {
            return;
        }

        // If the sink is empty, load the current track.
        if self.sink.empty() {
            let queue_item = self.queue[self.current_queue_item_index].clone();
            let _release = queue_item.release;
            let track = queue_item.track;
            let librarian = self.librarian.clone();
            let sink = self.sink.clone();
            // TODO i don't think the play below is safe, cause what if this
            // hasn't started downloading yet when it runs?
            // Probably time to figure out caching of songs.
            std::thread::spawn(move || {
                librarian.stream(&track, &sink).unwrap();
            });
            // TODO stopping here, tired. playing with preloading the next track.
            // if self.current_track_index < self.queue.len() - 1 {
            //     let next_track = self.queue[self.current_track_index + 1].clone();
            //     self.librarian.stream(&next_track, &self.sink).unwrap();
            // }
        }
        
        // And play it.
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn next(&mut self) {
        // If the playlist is empty, do nothing.
        if self.queue.is_empty() {
            return;
        }

        // Increment or restart the queue
        self.current_queue_item_index = (self.current_queue_item_index + 1) % self.queue.len();

        // If we were already playing, stop and play the new track
        if !self.sink.empty() {
            self.sink.clear();
            self.sink.sleep_until_end();
            self.play();
        }
    }

    pub fn previous(&mut self) {
        // If the playlist is empty, do nothing.
        if self.queue.is_empty() {
            return;
        }

        // Decrement or restart the queue
        if self.current_queue_item_index == 0 {
            self.current_queue_item_index = self.queue.len() - 1;
        }
        else {
            self.current_queue_item_index -= 1;
        }

        // If we were already playing, stop and play the new track
        if !self.sink.empty() {
            self.sink.clear();
            self.sink.sleep_until_end();
            self.play();
        }
    }

    pub fn clear(&mut self) {
        self.sink.stop();
        self.queue.clear();
        self.current_queue_item_index = 0;
    }

    pub fn duration(&self) -> f32 {
        367.8
    }

    pub fn position(&self) -> f32 {
        *self.position.read().unwrap()
    }

    pub fn seek(&self, position: f32) {
        *self.position.write().unwrap() = position;
    }
}

