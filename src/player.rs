use std::{sync::{Arc, RwLock}, time::Duration};

use rodio::Sink;

use crate::{music_library::{Track, Release, Library}, librarian::Librarian};

#[derive(Clone)]
pub struct Player {
    sink: Arc<Sink>,
    librarian: Arc<Librarian>,
    queue: Vec<QueueItem>,
    current_queue_item_index: usize,
}

#[derive(Clone, Debug)]
pub struct QueueItem {
    pub release: Release,
    pub track: Track,
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
        let player = Arc::new(RwLock::new(Self {
            sink,
            librarian,
            queue: Vec::new(),
            current_queue_item_index: 0,
        }));

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

        player
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
            self.librarian.stream(&track, &self.sink).unwrap();
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

    pub fn current_item(&self) -> Option<QueueItem> {
        if self.current_queue_item_index >= self.queue.len() {
            return None;
        }
        Some(self.queue[self.current_queue_item_index].clone())
    }

    pub fn next_item(&self) -> Option<QueueItem> {
        if self.current_queue_item_index + 1 >= self.queue.len() {
            return None;
        }
        Some(self.queue[self.current_queue_item_index + 1].clone())
    }

    pub fn clear(&mut self) {
        self.sink.stop();
        self.queue.clear();
        self.current_queue_item_index = 0;
    }
}

