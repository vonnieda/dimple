use std::collections::HashSet;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;
use crate::model::Release;
use crate::model::Entity;
use crate::model::ReleaseGroup;
use crate::model::Artist;
use crate::model::Recording;
use crate::collection::Collection;

use super::Entities;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MediaFile {
    pub key: Option<String>,
    pub url: String,
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
        let iter = col.list(&MediaFile::default().entity(), None)
            .map(|m| match m {
                Entities::MediaFile(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn entity(&self) -> Entities {
        Entities::MediaFile(self.clone())
    }
}

