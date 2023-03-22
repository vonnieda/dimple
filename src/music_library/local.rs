use std::io::Cursor;

use image::{ImageOutputFormat};
use serde::{Deserialize, Serialize};

/// A local music library living in a directory. Stores metadata in TBD
/// and media (art and music) in TBD. Currently exploring Sled.
/// 
/// This music library is how the app stores its local cache of all other
/// libraries and for that reason it is considered the reference implementation.

// TODO think about storing the images apart from the releases so that I can
// load images lazy, on a threadpool, and at different sizes.  Sled will make
// it super easy to cache at different sizes.

// TODO actually, now that I think of it... I should just use Sled for the
// image stuff in general. It seems to be working great.

use crate::MusicLibrary;

use super::Release;

pub struct LocalMusicLibrary {
    db: sled::Db,
}

impl LocalMusicLibrary {
    pub fn new(path: &str) -> Self {
        Self {
            db: sled::open(path).unwrap()
        }
    }
}

impl MusicLibrary for LocalMusicLibrary {
    fn releases(self: &Self) -> Vec<Release> {
        if let Ok(releases) = self.db.open_tree("releases") {
            let mut results = Vec::new();
            for release in releases.iter() {
                let bin = release.unwrap().1;
                let internal: InternalRelease = bincode::deserialize(&bin[..]).unwrap();
                let mut release = Release {
                    id: internal.id,
                    title: internal.title,
                    artist: internal.artist,
                    cover_image: None,
                };
                if let Ok(cover_image) = image::load_from_memory(&internal.cover_image) {
                    release.cover_image = Some(cover_image);
                }
                results.push(release);
            }
            return results;
        }
        
        Vec::new()
    }

    fn merge_release(self: &Self, release: &Release) -> Result<Release, String> {
        if let Ok(releases) = self.db.open_tree("releases") {
            let mut internal = InternalRelease {
                id: release.id.clone(),
                title: release.title.clone(),
                artist: release.artist.clone(),
                cover_image: Vec::new(),
            };
            if let Some(cover_image) = &release.cover_image {
                cover_image.write_to(&mut Cursor::new(&mut internal.cover_image), 
                    ImageOutputFormat::Png).expect("can't write png");
            }
            if let Ok(bin) = bincode::serialize(&internal) {
                releases.insert(&release.id, bin).expect("insert failed");
            }
        }
        return Ok(Release::default());
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct InternalRelease {
    id: String,
    title: String,
    artist: Option<String>,
    cover_image: Vec<u8>,
}