use serde::Deserialize;
use serde::Serialize;

use crate::collection::Collection;

use super::Entities;
use super::Entity;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MediaFile {
    pub key: String,

    // pub url: String,
    // pub created_at: Instant,
    // pub modified_at: Instant,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub title: Option<String>,
    pub genre: Option<String>,

    pub recording_mbid: Option<String>,
    pub release_track_mbid: Option<String>,
    pub album_mbid: Option<String>,
    pub artist_mbid: Option<String>,
    pub album_artist_mbid: Option<String>,
    pub mb_album_type: Option<String>,
    pub mb_album_comment: Option<String>,
}

impl MediaFile {
    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = MediaFile>> {
        let iter = col
            .list(&MediaFile::default().entity(), None)
            .map(|m| match m {
                Entities::MediaFile(m) => m,
                _ => panic!(),
            });
        Box::new(iter)
    }
}

impl Entity for MediaFile {
    fn key(&self) -> Option<String> {
        Some(self.key.clone())
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key.unwrap();
    }

    fn entity(&self) -> Entities {
        Entities::MediaFile(self.clone())
    }    
}
