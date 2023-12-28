// use std::{sync::Arc, path::{Path, PathBuf}, fs::{File}};

// use symphonia::{core::{codecs::CodecRegistry, probe::{Probe, Hint, ProbeResult}, io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions}, meta::{MetadataOptions, Tag, Value, MetadataRevision, Visual, ColorMode}, formats::{FormatOptions, Track, Cue}, units::TimeBase}, default};
// use walkdir::WalkDir;

// use super::{MusicLibrary, Release};

// use rayon::prelude::*;

use std::{path::Path, fs::File, collections::{hash_map, HashMap, HashSet}, time::Duration, io::Error, sync::{mpsc::{channel, Receiver}, RwLock}};

use audiotags::{Tag, AudioTag, AudioTagEdit, AudioTagConfig};
use dimple_core::{library::Library, model::{Release, Track, Artist, Genre}};
use image::DynamicImage;
use walkdir::{WalkDir, DirEntry};
use dimple_core::model::Image;

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

// This is all poop.

pub struct FolderLibrary {
    path: String,
    images_by_url: RwLock<HashMap<String, DynamicImage>>,
}

impl FolderLibrary {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            images_by_url: RwLock::new(HashMap::new()),
        }
    }

    fn url(&self, typ: &str, path: &str) -> String {
        format!("folder://{}/{}/{}", self.path, typ, path)
    }

    fn release(&self, tag: &dyn AudioTag) -> Release {
        // tag.album_artist().unwrap_or_default(),
        // tag.album_title().unwrap_or_default(),
        // tag.title().unwrap_or_default());

        Release {
            url: self.url("releases", tag.album_title().unwrap_or_default()),
            title: tag.album_title().unwrap_or_default().to_string(),
            artists: self.artists(tag),
            genres: self.genres(tag),
            art: self.art(tag),
            tracks: vec![self.track(tag)],
        }
    }

    fn track(&self, tag: &dyn AudioTag) -> Track {
        Track {
            // TODO better url
            url: self.url("tracks", tag.title().unwrap_or("")),
            artists: self.artists(tag),
            title: tag.title().unwrap_or("").to_string(),
            genres: self.genres(tag),
            art: vec![],
        }
    }

    fn artists(&self, tag: &dyn AudioTag) -> Vec<Artist> {
        vec![Artist {
            url: self.url("artists", tag.album_artist().unwrap_or("")),
            name: tag.album_artist().unwrap_or("").to_string(),
            art: vec![],
            genres: self.genres(tag),
        }]
    }

    fn genres(&self, tag: &dyn AudioTag) -> Vec<Genre> {
        if let Some(genre) = tag.genre() {
            return genre.split(';')
                .map(|genre| {
                    Genre {
                        url: self.url("genres", genre),
                        name: genre.to_string(),
                        art: vec![],
                    }
                })
                .collect();
        }
        vec![]
    }

    fn art(&self, tag: &dyn AudioTag) -> Vec<Image> {
        let path = format!("{}{}{}",
            tag.album_artist().unwrap_or_default(),
            tag.album_title().unwrap_or_default(),
            tag.title().unwrap_or_default());
        let url = self.url("images", &path);
        let image = tag.album_cover()
            .and_then(|album_cover| 
                image::load_from_memory(album_cover.data).ok());
        if let Some(image) = image {
            self.images_by_url.write().unwrap().insert(url.clone(), image.clone());
            log::debug!("Stored {}x{} image for {}", image.width(), 
                image.height(), url);
            return vec![Image {
                url,
            }];
        }
        vec![]        
    }

    fn merge_release(a: &Release, b: &Release) -> Release {
        let mut dest = b.clone();
        let src = a.clone();
        if dest.url.is_empty() {
            dest.url = src.url.clone();
        }
        if dest.title.is_empty() {
            dest.title = src.title.clone();
        }
        dest.artists = dest.artists.iter()
            .chain(src.artists.iter())
            .cloned()
            .collect::<HashSet<Artist>>()
            .iter()
            .cloned()
            .collect::<Vec<Artist>>();
        dest.genres = dest.genres.iter()
            .chain(src.genres.iter())
            .cloned()
            .collect::<HashSet<Genre>>()
            .iter()
            .cloned()
            .collect::<Vec<Genre>>();
        dest.art = dest.art.iter()
            .chain(src.art.iter())
            .cloned()
            .collect::<HashSet<Image>>()
            .iter()
            .cloned()
            .collect::<Vec<Image>>();
        dest.tracks = dest.tracks.iter()
            .chain(src.tracks.iter())
            .cloned()
            .collect::<HashSet<Track>>()
            .iter()
            .cloned()
            .collect::<Vec<Track>>();
        dest
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
            .map(|(e, tag)| {
                let tag = tag.unwrap();
                self.release(&*tag)
            })
            // TODO improve, probably just going to return tracks or hand
            // them off to a matching service.
            .fold(HashMap::new(), |mut acc, release| {
                if let Some(stored_release) = acc.get(&release.url) {
                    let release = Self::merge_release(&release, stored_release);
                    acc.insert(release.url.clone(), release);
                }
                else {
                    acc.insert(release.url.clone(), release);
                }
                acc
            })
            .values()
            .cloned()
            .for_each(|release| {
                log::debug!("Sending {:?}", &release);
                sender.send(release).unwrap();
            });

        receiver
    }

    fn image(&self, image: &dimple_core::model::Image) -> Result<DynamicImage, String> {
        if let Some(cached_image) = self.images_by_url.read().unwrap().get(&image.url) {
            log::debug!("Loaded {}x{} image for {}", cached_image.width(), 
            cached_image.height(), image.url);
            return Ok(cached_image.clone());
        }
        Err("not found".to_string())
    }

    fn stream(&self, _track: &dimple_core::model::Track) -> Result<Vec<u8>, String> {
        todo!()
    }
}
