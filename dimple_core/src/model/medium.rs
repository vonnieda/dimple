use serde::Deserialize;
use serde::Serialize;

use crate::model::Track;

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
