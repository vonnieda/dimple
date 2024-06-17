use std::collections::HashSet;
use std::hash::Hash;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::ArtistCredit;
use super::Genre;
use super::KnownIds;

// https://musicbrainz.org/doc/Recording
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
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

impl Hash for Recording {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.title.hash(state);
        self.known_ids.hash(state);
        self.disambiguation.hash(state);
        self.summary.hash(state);
        // self.links.hash(state);
        self.annotation.hash(state);
        self.length.hash(state);
        self.isrc.hash(state);
        self.artist_credits.hash(state);
        self.genres.hash(state);
    }
}