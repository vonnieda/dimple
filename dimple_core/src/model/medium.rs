use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

// https://musicbrainz.org/doc/Medium
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Medium {
    pub key: Option<String>,

    pub title: Option<String>,

    pub disc_count: Option<u32>,
    pub format: Option<String>,
    pub position: Option<u32>,
    pub track_count: Option<u32>,
}
