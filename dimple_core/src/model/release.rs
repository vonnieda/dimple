use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use super::KnownId;
use super::Model;

// https://musicbrainz.org/doc/Release
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Release {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub barcode: Option<String>,
    pub country: Option<String>,
    pub date: Option<String>, // TODO should be chronos, probably.
    pub packaging: Option<String>,
    pub status: Option<String>,
}

impl From<Release> for Model {
    fn from(value: Release) -> Self {
        Self::Release(value)
    }
}

impl From<Model> for Release {
    fn from(value: Model) -> Self {
        match value {
            Model::Release(value) => value,
            _ => panic!(),
        }
    }
}
