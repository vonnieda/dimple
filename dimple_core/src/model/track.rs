use std::collections::HashSet;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;
use crate::model::Entity;
use crate::model::RecordingSource;
use crate::model::Artist;
use crate::model::Recording;
use crate::model::Genre;
use crate::collection::Collection;

// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Track {
    pub key: Option<String>,

    pub title: String,

    pub length: u32,
    pub number: String,
    pub position: u32,
    pub recording: Recording,
    pub sources:  Vec<RecordingSource>,
}


