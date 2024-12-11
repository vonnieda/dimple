Dump new things at the bottom, move things to the top to prioritize.

- [x] List sorting.
- Track detail musicbrainz info, first pass.
- Lyrics.
- Tantivy search.
- Change player bar to use player.on_change.
- [x] Figure out how I'm going to implement queueing controls on table rows.
- Show currently playing track as playing in queue.
- Clicking on a playbar item should take you to the item in the queue, from
  there you can click the item to go to details if you want.
- Bug: Music skips when switching from the Dimple window to another app, maybe
  other times too. https://docs.rs/thread-priority/latest/thread_priority/
  I think it's due to thread priority.
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
- OTCH: Text is too small.
  I did a short pass on this sort of. I had previously disabled the font
  settings somehow so I got those back and that made the text larger and
  more clear. I also increased the text side of the sidebar items, and
  spaced them out further.
- Bug: Dragging the seek slider breaks the binding, so it stops updating
  after the first drag. 
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