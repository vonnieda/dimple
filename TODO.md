Dump new things at the bottom, move things to the top to prioritize.

- After starting the app, if there is music in the queue, pressing play on the
  keyboard should start playing. Right now it seems macos doesn't forward the
  play event until we've actually played at least once.
- Search zip files for importable items.
- Turn import into one scanner task and many processors task. You just import a
  file or directory and Dimple will find music, metadata, playlists, Spotify
  history, Apple Music history, etc.
- Double click in queue to start playing, inclusing starting the player. Single
  click is just for select.
- Bug: Sorting by ordinal is lex not numeric. 
- TODO an indicator in a playlist (or queue) that is dynamic and shows the
  delta between two songs based on a sum of stats. So like, you could tell
  at a glance when a playlist is going to switch up. I guess maybe if the 
  playlist had the moodbar or whatever and you could scroll them that would
  be insanely baller. They could be images, stored like images, handled like
  images. Or maybe "images" and actually what we store is a serialized vector
  so it scales nice. These will be plugins like "generate_audiogram(Song)" 
  so I can make a ton of them. This will be awesome cause we'll use the same
  code in the scrubber.
- Add a changelist page for debug and add it to settings.
- Mac menubar: https://mrmekon.github.io/fruitbasket/fruitbasket/,
  https://github.com/rust-windowing/winit/issues/1855
- bug: I think the last track in the queue is not getting scrobbled after finish
  playing.
- Looks like you can get and set window position now, so I can start saving
  that in config.
- Bug: Remove all from queue while playing removes the info from the play bar
  but music keeps playing.
- Bug: When going "Back" scroll position should be remembered. For example, as
  of 12/10/2024 if you go to tracks, then scroll down, then click a track, then
  click back, when you get back to track the scroll position is lost.
- When I pause the music to watch a video, and I have to turn the volume all
  the way up to hear the video, and then I unpause the music the music is of
  course far too loud. Might be nice to fade it back in or something if the
  volume has been adjusted by more than like 50% since it was paused.
- Needs to be easier to clear the editing of a click in text field like
  playlist name. Right now you have to hit enter - want at least escape or
  clicking outside of the field too.
- register the dimple:// handler with the OS so that those open Dimple and then
  I'll handle it in navigate. I'd like to be able to demo to people with:
  1. Download and run Dimple
  2. Click this link (dimple://share/???)
  This will open Dimple and go to a page showing metadata about the share, which
  in this case is a library of free music hosted on my B2 account. The user can
- Clicking playlist name to edit should focus and select all
- Bug: Empty queue shows placeholder text. Probably should just be empty.
- Need like, download, etc. buttons ont he player bar. Probably over near the
  play buttons.
- ReplayGain.
- Per song EQ.
- Bug: Clicking on a track in tracks with no title (or artist album) crashes.
- Bug: Margot and the Nuclear So and So's Vampires and Blue Dresses doesn't play
  and player just sits there.
- Bug: Moving a file on the filesystem leaves an orphaned and incorrect
  MediaFile behind.
- Re-combine all the grid pages into one relatively smart "results list", 
  that also has tags, sorting, filtering, etc.
- Each image on a page should be a slideshow, and the user can drop new images,
  and they can hide ones they don't like, and lock to a specific one.
- Drag and drop playlist management (and queue)
- album art in tables
- links in tables
- gonna have to write my own table
- retain scroll position on back and maybe in general if the url of a detail page doesn't change
- genre numbers imported instead of names
- auto refresh of media files
- importing is blowing up the ui due to too many notifications
- musicbrainz genres are not getting linked in import I think
- add support for composer: https://github.com/navidrome/navidrome/issues/211
- playlists should take releases and playlists as items
- number of plays on artists, tracks, releases, etc.
- popularity (from your history) on artists, tracks, releases, etc.

- m4a not playing (Built to Spill - You In Reverse)
    thread '<unnamed>' panicked at dimple_core/src/player/track_downloader.rs:41:90:
    called `Result::unwrap()` on an `Err` value: malformed stream: isomp4: overread atom

    Location:
        dimple_core/vendor/playback-rs/src/lib.rs:816:40
    stack backtrace:
      0: rust_begin_unwind
                at /rustc/90b35a6239c3d8bdabc530a6a0816f7ff89a0aaf/library/std/src/panicking.rs:665:5
      1: core::panicking::panic_fmt
                at /rustc/90b35a6239c3d8bdabc530a6a0816f7ff89a0aaf/library/core/src/panicking.rs:74:14
      2: core::result::unwrap_failed
                at /rustc/90b35a6239c3d8bdabc530a6a0816f7ff89a0aaf/library/core/src/result.rs:1700:5
      3: core::result::Result<T,E>::unwrap
                at /Users/jason/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:1104:23
      4: dimple_core::player::track_downloader::TrackDownloader::get::{{closure}}::{{closure}}
                at /Users/jason/Projects/Dimple/dimple_core/src/player/track_downloader.rs:41:28
      5: <F as threadpool::FnBox>::call_box
                at /Users/jason/.cargo/registry/src/index.crates.io-6f17d22bba15001f/threadpool-1.8.1/src/lib.rs:95:9
      6: threadpool::spawn_in_pool::{{closure}}
                at /Users/jason/.cargo/registry/src/index.crates.io-6f17d22bba15001f/threadpool-1.8.1/src/lib.rs:769:17
    note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

- architecture: the image lazy_get is index based, which has always been a race 
  condition and also just gross. Should be by key, or callback, or something else.
- Clicking the play bar image to go to the current song in the queue does not
  always work. I think this is a Slint issue.
- Menu positioning is still wrong on grids in a scroller, cause it doesn't
  take the scroll position into account. Maybe needs to move up and we have
  like a "card event" or something on card grid.