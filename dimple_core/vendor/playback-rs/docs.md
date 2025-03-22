`playback-rs` is a very, _very_ simple library to handle playing audio files using [`symphonia`] and [`cpal`], as well as [`rubato`] for sample rate conversion.

# How to use
To decode a song you can call [Song::new()] or [Song::from_file()] to get a [Song], which contains the uncompressed audio data stored in an [Arc] for lightweight cloning.

To play a song you can create a [Player], which will allow you to play [Song]s, as well as queueing a second songs to allow true gapless playback.
Once you have created the player you can play a song using [Player::play_song_next], but be sure to call [Player::has_next_song] or else you will overwrite that song in the queue.

After calling either method with a song that song may be discarded as the player retains an internal copy of the song data.

## The [Player]'s state
The [Player] struct contains an actively playing song as well as a single queued song that allows gapless playback.
Additionally, the player only ever loads songs into the queued song slot, never the actively playing song (though it can be cleared).
Because of this, you'll always want to call [Player::has_next_song] before calling [Player::play_song_next], to make sure you aren't overwriting another song you already queued.

The player also maintains a pause state, however this state is persistent outside any playing song, so calling [Player::set_playing] is the only way to modify this state.
While paused, the [Player] struct will output silence regardless of whether there is a song in the queue or not.

# Example
This example plays two songs and showcases quite a few of the utility functions provided by the [Player] struct.

```rust
use playback_rs::{Player, Song};

let player = Player::new(None).expect("Failed to open an audio output."); // Create a player to play audio with cpal.
let song = Song::from_file("song.mp3", None).expect("Failed to load or decode the song."); // Decode a song from a file
player.play_song_next(&song, Some(std::time::Duration::from_secs(30))).expect("Failed to play the song starting at 0:30");
let song2 = Song::from_file("song2.flac", None).expect("Failed to load or decode the other song.");
player.play_song_next(&song, None).expect("Failed to play the other song");

# #[cfg(not(doctest))]
std::thread::sleep(std::time::Duration::from_secs(5));
// Query playback position
println!("Playback position: {:?}", player.get_playback_position());

// Pause the song for a second
player.set_playing(false);
# #[cfg(not(doctest))]
std::thread::sleep(std::time::Duration::from_secs(1));
player.set_playing(true);
# #[cfg(not(doctest))]
std::thread::sleep(std::time::Duration::from_secs(5));
// Seek forward to 30 seconds
player.seek(std::time::Duration::from_secs(30));
# #[cfg(not(doctest))]
std::thread::sleep(std::time::Duration::from_secs(10));
// Skip to the second song
player.skip();

// Wait until the song has ended to exit
# #[cfg(not(doctest))]
while player.has_current_song() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
// Playback will automatically stop when the Player is dropped
```
