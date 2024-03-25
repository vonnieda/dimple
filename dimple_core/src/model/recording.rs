use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;

// https://musicbrainz.org/doc/Recording
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Recording {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub annotation: Option<String>,
    pub length: Option<u32>,

    pub isrcs: HashSet<String>,
}
