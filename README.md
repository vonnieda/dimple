# Dimple Music Player

Dimple is a music streaming platform in an app. It is local first, open source,
private, and cross-platform. With Dimple you can listen to your music and
playlists on every supported device, follow favorite artists, discover new
music and playlists every day, and a lot more! 

If you like Dimple, and would like to help me keep working on it, please
consider purchasing from one of the links below. Dimple is my full time job
and source of income, and every dollar helps me working on open source
software full time:

- Merch: https://dimple.lol/store
- BTC: 3FMNgdEjbVcxVoAUtgFFpzsuccnU9KMuhx
- ETH: 0xf1CE557bE8645dC70e78Cbb601bAF2b3649169A0
- DOGE: DGNKBH3AN4pUnHs9ZNQpC42ABzJG4mVF3t
- Paypal: jason@vonnieda.org
- Github: https://github.com/vonnieda
- Venmo: @Jason-vonNieda-1
- Ko-Fi: https://ko-fi.com/vonnieda
- Buy Me a Coffee: https://www.buymeacoffee.com/vonnieda
- Patreon: (Pending)
- Something missing or wrong? <a href="mailto:jason@vonnieda.org">Let me know!</a>
- Want to sponsor me? <a href="mailto:jason@vonnieda.org">Get in touch!</a>

# Features
- Open Source: https://github.com/vonnieda/Dimple

## Cross Platform
- [x] macOS
- [x] Linux
- [x] Windows
- [ ] iOS
- [ ] Android
- [ ] CarPlay
- [ ] Android Auto
- [ ] Apple tvOS

## Privacy
- No ads, no accounts, no data collection, no tracking, no DRM, no telemetry,
  no bullshit.
- Everything is stored locally by default.
- Synchronized data is end-to-end encrypted.
- Works offline with your downloaded music.

## Sync
- Sync, stream, and download your music on any supported device, with any S3
  compatible storage service. 
- Dimple offers a cheap and built in option that helps pay for development,
  or use any S3 compatible storage you like. You can even self-host!
- Keep your likes, listens, playlists, metadata and more synchronized between
  your devices

## Rich Metadata
- Metadata from MusicBrainz, Wikipedia, ListenBrainz, Last.fm, Discogs, etc.
- Artwork from Cover Art Archive, fanart.tv, Wiki Commons, etc.
- Lyrics from Genius, Musixmatch, Muzikum, etc.
- Scrobble to Last.fm, Listenbrainz, Maloja, etc.

## "Smart"

*Ed: explain the feature, drop the silly names.*

- "Deep Scrobbling" keeps track of your volume adjustments, repeats, skips,
  scrubs, likes, dislikes, and dozens of other interactive data points. This
  powers features like Current Obsessions, Mood Radio, AI DJ, ReplayGain,
  Instant Mixes, and more.
- Instant Mixes are infinite playlists based on Artists, Albums, Tracks,
  Genres, Playlists, and more. Everything is clickable and everything can
  start an Instant Mix.
- AI DJ uses text to speech and a small, fine-tuned AI brain to give you your
  own personal DJ. It announces songs and mixes, and adjusts to your
  reactions.
- Current Obsessions lets you jump right back into your recent favorites.
- Mood Radio instantly matches your mood with an infinite playlist of whatever
  you're feeling right now.
- ReplayGain automatically adjusts audio volume based on community feedback
  and your own adjustments.
- Sing Along shows and syncs the lyrics of whatever music is playing around
  you. Share your screen with your friends and sing along!

## Modern and Beautiful
- Based on Google's Material Design, with customizable themes.
- Artwork forward with abstract generated defaults when none is available.
- Responsive on any screen size.
- Light and Dark Mode, optionally follows your OS choice.
- Interactive waveform scrubbers let you "see" the music.
  - Moodbar: https://en.wikipedia.org/wiki/Moodbar
  - Sparkline: https://simonrepp.com/faircamp/#release
  - Spectrogram: https://en.wikipedia.org/wiki/Spectrogram
  - Wave
- Visualizations from projectM?: https://github.com/projectM-visualizer/projectm
  - https://crates.io/crates/projectm
  - https://crates.io/crates/projectm-sys

## DRM-Free
- Dimple does not use or support DRM. See https://www.defectivebydesign.org/guide/audio
- Support artists by buying and streaming their music from Bandcamp, Soundcloud,
  iTunes Music, and more, all within Dimple.

## Ethical
- Uses only public and authorized sources of data and music.
- Attribution is captured and displayed with all relevant data.
- Supports artists by givers users access to artist preferred downloads,
  streaming, and merch.
- Respects caching and rate limits wherever requested. 
- TBD% of revenue goes to MetaBrainz, Wikimedia, and other public resources.

## Fast
- Written in Rust using Slint.
- Async network and file I/O.
- Extensive local caching for fast browsing, even offline.

## Dimple Cloud (Optional)
- One click S3 compatible storage.
- Privately and securely synchronize all your devices.
- End-to-end encryption means your library is private and safe.
- Fast, reliable, and worldwide. Based on Backblaze B2 Cloud Storage.
- $0.99 for 100GB per month, or a year for $8.99. Pay only for what you use.
  100GB stores roughly 30,000 songs.
- All features above are also available for free using any S3 compatible storage
  provider.

# Mantras
- If a song has an outro that runs into the intro of the next song, don't play
  it without playing the next one!
- Shuffle should never go from Megadeth to They Might Be Giants.

# Notes
- I'm writing Dimple as a way to learn Rust. If you see weird things in the code
  it's probably because I didn't know a better way. Please feel free to
  open a PR or Issue and let me know! I'm still learning, and I appreciate the
  help!

# Acknowledgements
- Slint UI
- Phosphor Icons
- Tantivy Search
- [Tools Fairy](https://toolsfairy.com/image-test/sample-jpg-files)
- https://github.com/freestrings/waveform
- https://github.com/jamsocket/fractional_index

