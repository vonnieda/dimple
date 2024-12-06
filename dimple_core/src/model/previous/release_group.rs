use std::collections::HashSet;
use std::hash::Hash;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::ArtistCredit;
use super::Genre;
use super::KnownIds;


// https://musicbrainz.org/doc/ReleaseGroup
// https://musicbrainz.org/ws/2/release-group/1b4f4b3c-ca01-37b7-af1d-3e37989f86ad?inc=aliases%2Bartist-credits%2Breleases&fmt=json
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, ModelSupport)]
#[serde(default)]
pub struct ReleaseGroup {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: KnownIds,
    pub disambiguation: Option<String>,
    pub annotation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub first_release_date: Option<String>,
pub primary_type: Option<String>,
    pub secondary_types: HashSet<String>,

    #[serde(skip)]
    pub artist_credits: Vec<ArtistCredit>,
    #[serde(skip)]
    pub genres: Vec<Genre>,
}

impl Hash for ReleaseGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.title.hash(state);
        self.known_ids.hash(state);
        self.disambiguation.hash(state);
        self.annotation.hash(state);
        self.summary.hash(state);
        // self.links.hash(state);
        self.first_release_date.hash(state);
        self.primary_type.hash(state);
        // self.secondary_types.hash(state);
        self.artist_credits.hash(state);
        self.genres.hash(state);
    }
}