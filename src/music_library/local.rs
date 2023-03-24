/// A local music library living in a directory. Stores data with Sled.
///
/// This music library is how the app stores its local cache of all other
/// libraries and for that reason it is considered the reference implementation.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::MusicLibrary;
use super::{Release, image_cache::ImageCache};

pub struct LocalMusicLibrary {
    db: sled::Db,
    images: ImageCache,
}

impl LocalMusicLibrary {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        let images = ImageCache::new(db.open_tree("images").unwrap());
        Self { db, images }
    }
}

impl MusicLibrary for LocalMusicLibrary {
    fn releases(&self) -> Vec<Release> {
        let internal_releases: Vec<InternalRelease> = self
            .db
            .open_tree("releases")
            .unwrap()
            .iter()
            .par_bridge()
            .map(|kv| {
                let (_key, bin) = kv.unwrap();
                bincode::deserialize(&bin[..]).unwrap()
            })
            .collect();

        // TODO I think this can be done further in parallel by doing the
        // deserialization here.
        let releases: Vec<Release> = internal_releases
            .par_iter()
            .map(|internal_release| Release {
                id: internal_release.id.clone(),
                title: internal_release.title.clone(),
                artist: internal_release.artist.clone(),
                cover_art: self.images.get(&internal_release.id, 200, 200),
                genre: internal_release.genre.clone(),
                tracks: Vec::new(),
            })
            .collect();

        releases
    }

    // TODO change this to merge_releases and I can use Rayon here.
    fn merge_release(self: &Self, release: &Release) -> Result<Release, String> {
        if let Ok(releases) = self.db.open_tree("releases") {
            // Store the original cover image
            if let Some(cover_image) = &release.cover_art {
                // TODO error checking
                self.images.insert(&release.id, &cover_image);
            }

            // Create a serializable release
            let internal: InternalRelease = InternalRelease::from(release);

            // Store the release
            if let Ok(bin) = bincode::serialize(&internal) {
                releases.insert(&release.id, bin).expect("insert failed");
            }
        }
        // TODO return hydrated object
        return Ok(Release::default());
    }
}

impl From<&Release> for InternalRelease {
    fn from(release: &Release) -> Self {
        InternalRelease {
            id: release.id.clone(),
            title: release.title.clone(),
            artist: release.artist.clone(),
            cover_image_id: Some(release.id.clone()),
            genre: release.genre.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct InternalRelease {
    id: String,
    title: String,
    artist: Option<String>,
    cover_image_id: Option<String>,
    genre: Option<String>,
}
