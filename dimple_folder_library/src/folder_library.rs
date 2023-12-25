// use std::{sync::Arc, path::{Path, PathBuf}, fs::{File}};

// use symphonia::{core::{codecs::CodecRegistry, probe::{Probe, Hint, ProbeResult}, io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions}, meta::{MetadataOptions, Tag, Value, MetadataRevision, Visual, ColorMode}, formats::{FormatOptions, Track, Cue}, units::TimeBase}, default};
// use walkdir::WalkDir;

// use super::{MusicLibrary, Release};

// use rayon::prelude::*;

use std::{path::Path, fs::File, collections::{hash_map, HashMap}, time::Duration, io::Error, sync::mpsc::{channel, Receiver}};

use audiotags::{Tag, AudioTag};
use dimple_core::{library::Library, model::{Release, Track, Artist}};
use image::DynamicImage;
use walkdir::{WalkDir, DirEntry};

// TODO remember the idea of "you own this percent of your library" meaning
// you can have songs in your library that you maybe can't listen to, but
// want to keep track of. Maybe you can sample them, or listen on another site,
// but you don't have a file for them. This is how we can import scrobbles and
// playlists.
// So, the goal with any new piece of media should be to try to get it matched
// to a a standard database. MusicBrainz, or the internal, I suppose. But if
// it isn't matched, yet, it can still exist. 
// I think we have a many to many between types of IDs. We're gonna have metadata
// sources and stores and whatever, and each will have their own IDs but might
// also have their own additional IDs we can match against to try to create
// links or merges.

pub struct FolderLibrary {
    path: String,
}

impl FolderLibrary {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    fn base_url(&self) -> String {
        format!("folder:///{}", self.path)
    }
}

// https://github.com/diesel-rs/diesel
impl Library for FolderLibrary {
    fn name(&self) -> String {
        format!("FolderLibrary({})", self.path)
    }

    fn releases(&self) -> Receiver<Release> {
        let (sender, receiver) = channel::<Release>();
        WalkDir::new(self.path.clone())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some())
            .map(|e| (e.clone(), Tag::new().read_from_path(e.path())))
            .filter(|(_e, tag)| tag.is_ok())
            // Convert (DirEntry, Result<AudioTag>) to Release
            .map(|(e, tag)| {
                let tag = tag.unwrap();
                Release {
                    url: format!("{}/releases/{}", 
                        self.base_url(), 
                        tag.album_title().unwrap_or("")),
                    title: tag.album_title().unwrap_or("").to_string(),
                    tracks: vec![
                        Track {
                            url: format!("{}/tracks/{}",                         
                                self.base_url(), 
                                tag.title().unwrap_or("")),
                            title: tag.title().unwrap_or("").to_string(),
                            art: vec![],
                            artists: tag.artists()
                                .unwrap_or_default()
                                .iter()
                                .map(|artist_name| {
                                    Artist {
                                        url: format!("{}/artists/{}", self.base_url(), artist_name),
                                        name: artist_name.to_string(),
                                        art: vec![],
                                        genres: vec![],
                                    }
                                })
                                .collect(),
                            genres: vec![],
                        }
                    ],
                    art: vec![],
                    artists: tag.artists()
                        .unwrap_or_default()
                        .iter()
                        .map(|artist_name| {
                            Artist {
                                url: format!("{}/artists/{}", self.base_url(), artist_name),
                                name: artist_name.to_string(),
                                art: vec![],
                                genres: vec![],
                            }
                        })
                        .collect(),
                    genres: vec![],
                }
            })
            .for_each(|release| {
                log::info!("Sending {:?}", &release);
                sender.send(release).unwrap();
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



            // // Merge similar releases together by URL
            // .fold(HashMap::new(), |mut acc, rel_a| {
            //     if let rel_b = acc.get(&rel_a.url) {
            //         // merge em
            //     }
            //     let rel_b = acc.entry(release_url.clone()).or_insert(release);
            //     // release.artists.extend_from_slice(&e.track.artists);
            //     // release.tracks.push(e.track);
            //     // release.art.extend_from_slice(reease)
            //     acc
            // })
            // .values()
            // .cloned()
