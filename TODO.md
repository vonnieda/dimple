Dump new things at the bottom, move things to the top to prioritize.

- [x] List sorting.
- Track detail musicbrainz info, first pass.
- Lyrics.
- Tantivy search.
- Change player bar to use player.on_change.
- Figure out how I'm going to implement queueing controls on table rows.
- Show currently playing track as playing in queue.
- Bug: Music skips when switching from the Dimple window to another app, maybe
  other times too.
- After starting the app, if there is music in the queue, pressing play on the
  keyboard should start playing. Right now it seems macos doesn't forward the
  play event until we've actually played at least once.
- Search zip files for importable items.
- Turn import into one scanner task and many processors task. You just import a
  file or directory and Dimple will find music, metadata, playlists, Spotify
  history, Apple Music history, etc.
- Double click in queue to start playing, inclusing starting the player. Single
  click is just for select.