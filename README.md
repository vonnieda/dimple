# Dimple Music Player

Dimple is a local-first, cross-platform music player for people who want the
convenience of a music streaming service without all the 
[ick](https://en.wikipedia.org/wiki/Enshittification). I'm writing Dimple to
replace music streaming services in my life with privacy respecting software
that is built to last.

<div align="center">
  <a href="./assets/images/"><img src="assets/images/Screenshot%202025-03-30%20at%204.41.04 PM.png" width="100%"></a>
</div>

If you like Dimple, and would like to help me keep working on it, please
consider helping with one of the methods below. Dimple is my full time job
and only source of income, and every single bit helps me keep working on 
open source software full time:

- BTC: 3FMNgdEjbVcxVoAUtgFFpzsuccnU9KMuhx
- ETH: 0xf1CE557bE8645dC70e78Cbb601bAF2b3649169A0
- DOGE: DGNKBH3AN4pUnHs9ZNQpC42ABzJG4mVF3t
- Paypal: jason@vonnieda.org
- Github: https://github.com/vonnieda
- Venmo: @Jason-vonNieda-1
- Ko-Fi: https://ko-fi.com/vonnieda
- Buy Me a Coffee: https://www.buymeacoffee.com/vonnieda
- Patreon: (Pending)
- Merch: (Pending)
- Something missing or wrong? <a href="mailto:jason@vonnieda.org">Let me know!</a>
- Want to sponsor me? <a href="mailto:jason@vonnieda.org">Get in touch!</a>

# Status

Dimple is currently under heavy development and is **ALPHA QUALITY SOFTWARE**.
Everything is subject to change, and you should back up your database regularly
if you use it. There are several known issues, and you should expect to run
into bugs and crashes. Many features that are shown in the UI do not work yet.

I am working towards a 1.0 release after which the data model will be stable
and only modified through migrations. 

If you would like to try Dimple now you can download the source and run it
with `cargo run --release --bin dimple_ui_slint`.

Prebuilt binaries for macOS and Windows are available in 
[Actions](https://github.com/vonnieda/dimple/actions) under assets.

# How Does It Work?

Dimple plays your local music files and can stream from online and self-hosted
music services using plugins.

It keeps your music, images, metadata, and listening history in a Library, which
is a SQLite database. Dimple can sync your Library between all your devices
with end-to-end encryption.

As you use Dimple it searches public music databases and sites for new
information like events, lyrics, artwork, new releases, etc. and adds
it to your Library. 

Dimple uses plugins to perform many tasks, and you can write your own plugins
to extend Dimple and add new functionality.

# Features

- [x] Cross-platform desktop music player for macOS, Linux, and Windows. Mobile
  coming later.
- [x] Stores music, artwork, and metadata locally so it always works offline.
- [ ] Sync, stream, and download your music on any supported device, with any [S3
  compatible storage service](https://www.s3compare.io/), including self-hosted
  ones.
- [x] Personalized recommendations based on your listening history, stored and
  processed locally.
- [x] Artwork and metadata from popular public music databases like MusicBrainz,
  fanart.tv, and TheAudioDB. More being added via plugins!
- [x] Fast and responsive UI built with Rust and Slint.
- [x] Waveform and spectrograph scrubbers.
- [ ] Playlist management.
- [ ] Timestamp reactions, notes, and emoji.
- [x] Lyrics.
- [ ] Synchronized lyrics.
- [ ] ReplayGain.

# Platform Roadmap

- Desktop App (In Progress)
  - [ ] macOS
  - [ ] Linux
  - [ ] Windows
- Mobile App
  - [ ] iOS
  - [ ] Android
- Car App
  - [ ] CarPlay
  - [ ] Android Auto
- Home App
  - [ ] Apple tvOS

See [TODO.md](TODO.md) for additional backlog and wish list items.

# Development

I'm writing Dimple as a way to learn Rust. If you see weird things in the code
it's probably because I didn't know a better way. Please feel free to
<a href="mailto:jason@vonnieda.org">email me</a> and let me know! I'm still
learning, and I appreciate the help!

I'm not currently accepting PRs or new Issues while I focus on getting a v1.0
finished. Much of the architecture of the app is still in flux and I want to
focus on the app for now.

# Greetings and Thanks

- [MusicBrainz](https://musicbrainz.org/)
- [Slint UI](https://slint.dev/)
- [Phosphor Icons](https://phosphoricons.com/)
- [Tools Fairy](https://toolsfairy.com/image-test/sample-jpg-files)
- https://github.com/freestrings/waveform
- https://github.com/jamsocket/fractional_index
- https://github.com/RustyNova016/musicbrainz_rs

