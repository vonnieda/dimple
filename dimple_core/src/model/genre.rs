use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Genre {
    pub key: Option<String>,
    pub name: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,
}
