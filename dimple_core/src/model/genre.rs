use std::collections::HashSet;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::KnownIds;

// https://musicbrainz.org/doc/Genre
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Genre {
    pub key: Option<String>,
    pub name: Option<String>,
    pub known_ids: HashSet<KnownIds>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,
}
