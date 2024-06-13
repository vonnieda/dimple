use std::collections::HashSet;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::ArtistCredit;
use super::Genre;
use super::KnownIds;

// https://musicbrainz.org/doc/Recording
#[derive(Clone, Debug, Serialize, Deserialize, Default, ModelSupport)]
pub struct Recording {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: KnownIds,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub annotation: Option<String>,
    pub length: Option<u32>,

    pub isrc: Option<String>,

    #[serde(skip)]
    pub artist_credits: Vec<ArtistCredit>,
    #[serde(skip)]
    pub genres: Vec<Genre>,
}
