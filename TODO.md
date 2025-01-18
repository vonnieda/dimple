Dump new things at the bottom, move things to the top to prioritize.

- [x] List sorting.
- Track detail musicbrainz info, first pass.
- [x] Lyrics.
- Tantivy search.
- Change player bar to use player.on_change.
- [x] Figure out how I'm going to implement queueing controls on table rows.
- Show currently playing track as playing in queue.
- Clicking on a playbar item should take you to the item in the queue, from
  there you can click the item to go to details if you want.
- [x] Bug: Music skips when switching from the Dimple window to another app, maybe
  other times too. https://docs.rs/thread-priority/latest/thread_priority/
  I think it's due to thread priority.
  Okay, looks like this is fixed by cloning playback-rs, adding the thread
  priority crate and setting the playback thread to max. Haven't heard a skip
  since.
- After starting the app, if there is music in the queue, pressing play on the
  keyboard should start playing. Right now it seems macos doesn't forward the
  play event until we've actually played at least once.
- Search zip files for importable items.
- Turn import into one scanner task and many processors task. You just import a
  file or directory and Dimple will find music, metadata, playlists, Spotify
  history, Apple Music history, etc.
- Double click in queue to start playing, inclusing starting the player. Single
  click is just for select.
- [x] comment: not enough space between search and home.
- [x] OTCH: Text is too small.
  I did a short pass on this sort of. I had previously disabled the font
  settings somehow so I got those back and that made the text larger and
  more clear. I also increased the text side of the sidebar items, and
  spaced them out further.
- Bug: Dragging the seek slider breaks the binding, so it stops updating
  after the first drag. 
  https://github.com/slint-ui/slint/discussions/7120
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
- Per song EQ. ReplayGain is cool, but give the user *everything*.
- Bug: Clicking on a track in tracks with no title (or artist album) crashes.
- Bug: Margot and the Nuclear So and So's Vampires and Blue Dresses doesn't play
  and player just sits there.
- Bug: Moving a file on the filesystem leaves an orphaned and incorrect
  MediaFile behind.
- Make playlsit item ordinal real.
- Bug: Seeing a lot of lyrics with double newlines. 
  Ex: Blue Oyster Cult - Heavy Metal: The Black and Silver.
- Re-combine all the grid pages into one relatively smart "results list", 
  that also has tags, sorting, filtering, etc.
- Each image on a page should be a slideshow, and the user can drop new images,
  and they can hide ones they don't like, and lock to a specific one.
- Drag and drop playlist management (and queue)