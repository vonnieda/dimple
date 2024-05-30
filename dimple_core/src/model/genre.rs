use std::collections::HashSet;
use std::hash::Hash;

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

impl Hash for Genre {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.name.hash(state);
        // TODO hmmm... I'm mostly using tis for de-dupe, so it may make sense
        // to just use name and key.
        // self.known_ids.hash(state);
        self.disambiguation.hash(state);
        self.summary.hash(state);
        // self.links.hash(state);
    }    
}