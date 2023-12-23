// use std::{sync::Arc, path::{Path, PathBuf}, fs::{File}};

// use symphonia::{core::{codecs::CodecRegistry, probe::{Probe, Hint, ProbeResult}, io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions}, meta::{MetadataOptions, Tag, Value, MetadataRevision, Visual, ColorMode}, formats::{FormatOptions, Track, Cue}, units::TimeBase}, default};
// use walkdir::WalkDir;

// use super::{MusicLibrary, Release};

// use rayon::prelude::*;

use std::{path::Path, fs::File, collections::{hash_map, HashMap}};

use dimple_core::{library::Library, model::{Release, Track, Artist}};
use image::DynamicImage;
use symphonia::core::{io::{MediaSourceStream, MediaSource}, probe::{Hint, ProbeResult}, meta::{MetadataOptions, Visual, Tag, ColorMode, Value, StandardTagKey}, formats::FormatOptions, codecs::CodecRegistry};
use walkdir::WalkDir;

// TODO remember the idea of "you own this percent of your library" meaning
// you can have songs in your library that you maybe can't listen to, but
// want to keep track of. Maybe you can sample them, or listen on another site,
// but you don't have a file for them. This is how we can import scrobbles and
// playlists.
// So, the goal with any new piece of media should be to try to get it matched
// to a a standard database. MusicBrainz, or the internal, I suppose. But if
// it isn't matched, yet, it can still exist. 

pub struct FolderLibrary {
    path: String,
}

impl FolderLibrary {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    fn read_track(path: &Path) -> TrackInfo {
        log::info!("{}", path.to_str().unwrap());
        let file = File::open(path).unwrap();     

        let mut hint = Hint::default();
        if let Some(extension) = path.to_str().unwrap().split('.').last() {
            hint.with_extension(extension);
        }

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let media_source_stream = MediaSourceStream::new(
            Box::new(file) as Box<dyn MediaSource>, 
            Default::default());

        let track = Track { 
            url: path.to_str().unwrap().to_string(),
            title: "".to_string(), 
            art: vec![], 
            artists: vec![], 
            genres: vec![], 
        };
        let mut track_info = TrackInfo { 
            track, 
            album: "".to_string(),
            album_artist: "".to_string(),
        };

        let probe = symphonia::default::get_probe();
        if let Ok(mut probe_results) = probe.format(&hint, media_source_stream, &fmt_opts, &meta_opts) {
            if let Some(metadata_rev) = probe_results.format.metadata().current() {
                for tag in metadata_rev.tags() {
                    if tag.is_known() {
                        match tag.std_key {
                            Some(StandardTagKey::TrackTitle) => track_info.track.title = tag.value.to_string(),
                            Some(StandardTagKey::Artist) => {
                                let artist = Artist {
                                    url: format!("//artists/{}", tag.value),
                                    name: tag.value.to_string(),
                                    art: vec![],
                                    genres: vec![],
                                };
                                track_info.track.artists.push(artist);
                            },
                            Some(StandardTagKey::Album) => track_info.album = tag.value.to_string(),
                            Some(StandardTagKey::AlbumArtist) => track_info.album_artist = tag.value.to_string(),
                            _ => {}
                        }
                    }
                }
            }
        }

        track_info
    }
}

struct TrackInfo {
    track: Track,
    album: String,
    album_artist: String,
}

// https://github.com/diesel-rs/diesel
impl Library for FolderLibrary {
    fn name(&self) -> String {
        format!("FolderLibrary({})", self.path)
    }

    fn releases(&self) -> std::sync::mpsc::Receiver<dimple_core::model::Release> {
        // N + 1 Add Caching
        // 1 Get a list of all the files in the directory recursively
        // 2 For each one, scan it to see if it is a supported file
        // 3 Read tags on each supported file, creating Tracks, I think?
        // 4 Then, I suppose, we try to sort those into releases? Or build them
        //   into a hash of releases as we go?

        let (sender, receiver) = std::sync::mpsc::channel::<Release>();
        // let base_url = self.base_url();
        
        let s = self.path.clone();
        std::thread::spawn(move || {
            let path = Path::new(&s);
            let walkdir = WalkDir::new(path);
            walkdir.into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| Self::read_track(e.path()))
                .fold(HashMap::new(), |mut acc, e| {
                    let release_url = format!("//{}/{}", e.album_artist, e.album);
                    let release = acc.entry(release_url.clone()).or_insert(Release {
                        url: release_url,
                        title: e.album,
                        ..Default::default()    
                    });
                    release.artists.extend_from_slice(&e.track.artists);
                    release.tracks.push(e.track);
                    acc
                })
                .values()
                .cloned()
                .for_each(|release| {
                    sender.send(release).unwrap();
                });
        });

        receiver
    }

    fn image(&self, _image: &dimple_core::model::Image) -> Result<DynamicImage, String> {
        Result::Err("not yet implemented".to_string())
    }

    fn stream(&self, _track: &dimple_core::model::Track) -> Result<Vec<u8>, String> {
        todo!()
    }
}

// fn print_format(path: &str, probed: &mut ProbeResult) {
//     println!("+ {}", path);
//     // print_tracks(probed.format.tracks());

//     // Prefer metadata that's provided in the container format, over other tags found during the
//     // probe operation.
//     if let Some(metadata_rev) = probed.format.metadata().current() {
//         print_tags(metadata_rev.tags());
//         print_visuals(metadata_rev.visuals());

//         // Warn that certain tags are preferred.
//         if probed.metadata.get().as_ref().is_some() {
//             println!("tags that are part of the container format are preferentially printed.");
//             println!("not printing additional tags that were found while probing.");
//         }
//     }
//     else if let Some(metadata_rev) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
//         print_tags(metadata_rev.tags());
//         print_visuals(metadata_rev.visuals());
//     }

//     // print_cues(probed.format.cues());
//     println!(":");
//     println!();
// }

// // fn print_update(rev: &MetadataRevision) {
// //     print_tags(rev.tags());
// //     print_visuals(rev.visuals());
// //     println!(":");
// //     println!();
// // }

// // fn print_tracks(tracks: &[Track]) {
// //     if !tracks.is_empty() {
// //         println!("|");
// //         println!("| // Tracks //");

// //         for (idx, track) in tracks.iter().enumerate() {
// //             let params = &track.codec_params;

// //             print!("|     [{:0>2}] Codec:           ", idx + 1);

// //             if let Some(codec) = symphonia::default::get_codecs().get_codec(params.codec) {
// //                 println!("{} ({})", codec.long_name, codec.short_name);
// //             }
// //             else {
// //                 println!("Unknown (#{})", params.codec);
// //             }

// //             if let Some(sample_rate) = params.sample_rate {
// //                 println!("|          Sample Rate:     {}", sample_rate);
// //             }
// //             if params.start_ts > 0 {
// //                 if let Some(tb) = params.time_base {
// //                     println!(
// //                         "|          Start Time:      {} ({})",
// //                         fmt_time(params.start_ts, tb),
// //                         params.start_ts
// //                     );
// //                 }
// //                 else {
// //                     println!("|          Start Time:      {}", params.start_ts);
// //                 }
// //             }
// //             if let Some(n_frames) = params.n_frames {
// //                 if let Some(tb) = params.time_base {
// //                     println!(
// //                         "|          Duration:        {} ({})",
// //                         fmt_time(n_frames, tb),
// //                         n_frames
// //                     );
// //                 }
// //                 else {
// //                     println!("|          Frames:          {}", n_frames);
// //                 }
// //             }
// //             if let Some(tb) = params.time_base {
// //                 println!("|          Time Base:       {}", tb);
// //             }
// //             if let Some(padding) = params.delay {
// //                 println!("|          Encoder Delay:   {}", padding);
// //             }
// //             if let Some(padding) = params.padding {
// //                 println!("|          Encoder Padding: {}", padding);
// //             }
// //             if let Some(sample_format) = params.sample_format {
// //                 println!("|          Sample Format:   {:?}", sample_format);
// //             }
// //             if let Some(bits_per_sample) = params.bits_per_sample {
// //                 println!("|          Bits per Sample: {}", bits_per_sample);
// //             }
// //             if let Some(channels) = params.channels {
// //                 println!("|          Channel(s):      {}", channels.count());
// //                 println!("|          Channel Map:     {}", channels);
// //             }
// //             if let Some(channel_layout) = params.channel_layout {
// //                 println!("|          Channel Layout:  {:?}", channel_layout);
// //             }
// //             if let Some(language) = &track.language {
// //                 println!("|          Language:        {}", language);
// //             }
// //         }
// //     }
// // }

// // fn print_cues(cues: &[Cue]) {
// //     if !cues.is_empty() {
// //         println!("|");
// //         println!("| // Cues //");

// //         for (idx, cue) in cues.iter().enumerate() {
// //             println!("|     [{:0>2}] Track:      {}", idx + 1, cue.index);
// //             println!("|          Timestamp:  {}", cue.start_ts);

// //             // Print tags associated with the Cue.
// //             if !cue.tags.is_empty() {
// //                 println!("|          Tags:");

// //                 for (tidx, tag) in cue.tags.iter().enumerate() {
// //                     if let Some(std_key) = tag.std_key {
// //                         println!(
// //                             "{}",
// //                             print_tag_item(tidx + 1, &format!("{:?}", std_key), &tag.value, 21)
// //                         );
// //                     }
// //                     else {
// //                         println!("{}", print_tag_item(tidx + 1, &tag.key, &tag.value, 21));
// //                     }
// //                 }
// //             }

// //             // Print any sub-cues.
// //             if !cue.points.is_empty() {
// //                 println!("|          Sub-Cues:");

// //                 for (ptidx, pt) in cue.points.iter().enumerate() {
// //                     println!(
// //                         "|                      [{:0>2}] Offset:    {:?}",
// //                         ptidx + 1,
// //                         pt.start_offset_ts
// //                     );

// //                     // Start the number of sub-cue tags, but don't print them.
// //                     if !pt.tags.is_empty() {
// //                         println!(
// //                             "|                           Sub-Tags:  {} (not listed)",
// //                             pt.tags.len()
// //                         );
// //                     }
// //                 }
// //             }
// //         }
// //     }
// // }

// fn print_tags(tags: &[Tag]) {
//     if !tags.is_empty() {
//         println!("|");
//         println!("| // Tags //");

//         let mut idx = 1;

//         // Print tags with a standard tag key first, these are the most common tags.
//         for tag in tags.iter().filter(|tag| tag.is_known()) {
//             if let Some(std_key) = tag.std_key {
//                 println!("{}", print_tag_item(idx, &format!("{:?}", std_key), &tag.value, 4));
//             }
//             idx += 1;
//         }

//         // Print the remaining tags with keys truncated to 26 characters.
//         for tag in tags.iter().filter(|tag| !tag.is_known()) {
//             println!("{}", print_tag_item(idx, &tag.key, &tag.value, 4));
//             idx += 1;
//         }
//     }
// }

// fn print_visuals(visuals: &[Visual]) {
//     if !visuals.is_empty() {
//         println!("|");
//         println!("| // Visuals //");

//         for (idx, visual) in visuals.iter().enumerate() {
//             if let Some(usage) = visual.usage {
//                 println!("|     [{:0>2}] Usage:      {:?}", idx + 1, usage);
//                 println!("|          Media Type: {}", visual.media_type);
//             }
//             else {
//                 println!("|     [{:0>2}] Media Type: {}", idx + 1, visual.media_type);
//             }
//             if let Some(dimensions) = visual.dimensions {
//                 println!(
//                     "|          Dimensions: {} px x {} px",
//                     dimensions.width, dimensions.height
//                 );
//             }
//             if let Some(bpp) = visual.bits_per_pixel {
//                 println!("|          Bits/Pixel: {}", bpp);
//             }
//             if let Some(ColorMode::Indexed(colors)) = visual.color_mode {
//                 println!("|          Palette:    {} colors", colors);
//             }
//             println!("|          Size:       {} bytes", visual.data.len());

//             // Print out tags similar to how regular tags are printed.
//             if !visual.tags.is_empty() {
//                 println!("|          Tags:");
//             }

//             for (tidx, tag) in visual.tags.iter().enumerate() {
//                 if let Some(std_key) = tag.std_key {
//                     println!(
//                         "{}",
//                         print_tag_item(tidx + 1, &format!("{:?}", std_key), &tag.value, 21)
//                     );
//                 }
//                 else {
//                     println!("{}", print_tag_item(tidx + 1, &tag.key, &tag.value, 21));
//                 }
//             }
//         }
//     }
// }

// fn print_tag_item(idx: usize, key: &str, value: &Value, indent: usize) -> String {
//     let key_str = match key.len() {
//         0..=28 => format!("| {:w$}[{:0>2}] {:<28} : ", "", idx, key, w = indent),
//         _ => format!("| {:w$}[{:0>2}] {:.<28} : ", "", idx, key.split_at(26).0, w = indent),
//     };

//     let line_prefix = format!("\n| {:w$} : ", "", w = indent + 4 + 28 + 1);
//     let line_wrap_prefix = format!("\n| {:w$}   ", "", w = indent + 4 + 28 + 1);

//     let mut out = String::new();

//     out.push_str(&key_str);

//     for (wrapped, line) in value.to_string().lines().enumerate() {
//         if wrapped > 0 {
//             out.push_str(&line_prefix);
//         }

//         let mut chars = line.chars();
//         let split = (0..)
//             .map(|_| chars.by_ref().take(72).collect::<String>())
//             .take_while(|s| !s.is_empty())
//             .collect::<Vec<_>>();

//         out.push_str(&split.join(&line_wrap_prefix));
//     }

//     out
// }

// // fn fmt_time(ts: u64, tb: TimeBase) -> String {
// //     let time = tb.calc_time(ts);

// //     let hours = time.seconds / (60 * 60);
// //     let mins = (time.seconds % (60 * 60)) / 60;
// //     let secs = f64::from((time.seconds % 60) as u32) + time.frac;

// //     format!("{}:{:0>2}:{:0>6.3}", hours, mins, secs)
// // }

// // // fn print_progress(ts: u64, dur: Option<u64>, tb: Option<TimeBase>) {
// // //     // Get a string slice containing a progress bar.
// // //     fn progress_bar(ts: u64, dur: u64) -> &'static str {
// // //         const NUM_STEPS: usize = 60;

// // //         lazy_static! {
// // //             static ref PROGRESS_BAR: Vec<String> = {
// // //                 (0..NUM_STEPS + 1).map(|i| format!("[{:<60}]", str::repeat("■", i))).collect()
// // //             };
// // //         }

// // //         let i = (NUM_STEPS as u64)
// // //             .saturating_mul(ts)
// // //             .checked_div(dur)
// // //             .unwrap_or(0)
// // //             .clamp(0, NUM_STEPS as u64);

// // //         &PROGRESS_BAR[i as usize]
// // //     }

// // //     // Multiple print! calls would need to be made to print the progress, so instead, only lock
// // //     // stdout once and use write! rather then print!.
// // //     let stdout = std::io::stdout();
// // //     let mut output = stdout.lock();

// // //     if let Some(tb) = tb {
// // //         let t = tb.calc_time(ts);

// // //         let hours = t.seconds / (60 * 60);
// // //         let mins = (t.seconds % (60 * 60)) / 60;
// // //         let secs = f64::from((t.seconds % 60) as u32) + t.frac;

// // //         write!(output, "\r\u{25b6}\u{fe0f}  {}:{:0>2}:{:0>4.1}", hours, mins, secs).unwrap();

// // //         if let Some(dur) = dur {
// // //             let d = tb.calc_time(dur.saturating_sub(ts));

// // //             let hours = d.seconds / (60 * 60);
// // //             let mins = (d.seconds % (60 * 60)) / 60;
// // //             let secs = f64::from((d.seconds % 60) as u32) + d.frac;

// // //             write!(output, " {} -{}:{:0>2}:{:0>4.1}", progress_bar(ts, dur), hours, mins, secs)
// // //                 .unwrap();
// // //         }
// // //     }
// // //     else {
// // //         write!(output, "\r\u{25b6}\u{fe0f}  {}", ts).unwrap();
// // //     }

// // //     // This extra space is a workaround for Konsole to correctly erase the previous line.
// // //     write!(output, " ").unwrap();

// // //     // Flush immediately since stdout is buffered.
// // //     output.flush().unwrap();
// // // }

// // extern crate symphonia;

// // use symphonia::core::io::{MediaSourceStream, MediaSource};
// // use symphonia::core::probe::Hint;
// // use symphonia::core::meta::Metadata;
// // use std::fs::File;

// // fn main() {
// //     let file = File::open("path_to_your_mp3_file.mp3").expect("Error opening file");
// //     let mss = MediaSourceStream::new(Box::new(file) as Box<dyn MediaSource>, Default::default());
// //     let hint = Hint::new();
// //     let probed = symphonia::default::get_probe().format(&hint, mss, &Default::default()).expect("Error probing format");
    
// //     let format = probed.format;
// //     let reader = format.into_reader();

// //     // Iterate over metadata
// //     for meta in reader.metadata() {
// //         match meta {
// //             Metadata::Id3v2(tag) => {
// //                 // Print basic tags
// //                 println!("Title: {:?}", tag.title());
// //                 println!("Artist: {:?}", tag.artist());
// //                 println!("Album: {:?}", tag.album());

// //                 // Handling artwork
// //                 for picture in tag.pictures() {
// //                     println!("Artwork MIME type: {}", picture.mime_type);
// //                     // You can save the image data or process it further
// //                 }
// //             },
// //             _ => {}
// //         }
// //     }
// // }



