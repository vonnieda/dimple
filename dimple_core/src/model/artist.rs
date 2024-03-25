use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;

use super::Model;

// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub country: Option<String>,
}

impl From<Artist> for Model {
    fn from(value: Artist) -> Self {
        Self::Artist(value)
    }
}

impl From<Model> for Artist {
    fn from(value: Model) -> Self {
        match value {
            Model::Artist(value) => value,
            _ => panic!(),
        }
    }
}
