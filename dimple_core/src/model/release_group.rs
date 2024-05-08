use std::collections::HashSet;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;


// https://musicbrainz.org/doc/ReleaseGroup
// https://musicbrainz.org/ws/2/release-group/1b4f4b3c-ca01-37b7-af1d-3e37989f86ad?inc=aliases%2Bartist-credits%2Breleases&fmt=json
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
#[serde(default)]
pub struct ReleaseGroup {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub first_release_date: Option<String>,
    pub primary_type: Option<String>,
}
