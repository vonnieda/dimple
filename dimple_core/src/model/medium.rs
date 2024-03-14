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
use crate::model::Track;
use crate::collection::Collection;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

// https://musicbrainz.org/doc/Medium
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Medium {
    pub key: Option<String>,

    pub title: String,

    pub disc_count: u32,
    pub format: String,
    pub position: u32,
    pub track_count: u32,
    pub tracks: Vec<Track>,
}


