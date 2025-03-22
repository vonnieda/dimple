# [playback-rs](https://docs.rs/playback-rs/) ![Crates.io](https://img.shields.io/crates/v/playback-rs) ![docs.rs](https://img.shields.io/docsrs/playback-rs)
`playback-rs` is a very, _very_ simple library to handle playing audio files using [Symphonia](https://docs.rs/symphonia/) and [cpal](https://docs.rs/symphonia/), as well as [rubato](https://docs.rs/rubato/) for sample rate conversion.
It was made for and is the library used by [kiku](https://gitlab.101100.ca/heards/kiku/).

## Run the Example
Symphonia is very slow when running in debug mode, so it is recommended to run the example in release mode:
```sh
cargo run --release --example=aplay -- song1.mp3 song2.flac song3.ogg
```
