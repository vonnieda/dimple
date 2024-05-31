use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Write;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::known_id::KnownIds;
use super::Genre;

// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub known_ids: KnownIds,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub country: Option<String>,
    
    #[serde(skip)]
    pub genres: Vec<Genre>,
}

