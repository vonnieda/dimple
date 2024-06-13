use std::collections::HashSet;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::ArtistCredit;
use super::Genre;
use super::KnownIds;
use super::Medium;
use super::ReleaseGroup;

// https://musicbrainz.org/doc/Release
// https://musicbrainz.org/release/a4864e94-6d75-4ade-bc93-0dabf3521453
// https://musicbrainz.org/ws/2/release/a4864e94-6d75-4ade-bc93-0dabf3521453?fmt=json
#[derive(Clone, Debug, Serialize, Deserialize, Default, ModelSupport)]
#[serde(default)]
pub struct Release {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: KnownIds,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub barcode: Option<String>,
    pub country: Option<String>,
    pub date: Option<String>,
    pub primary_type: Option<String>,
    pub packaging: Option<String>,
    pub status: Option<String>,
    pub quality: Option<String>,

    #[serde(skip)]
    pub artist_credits: Vec<ArtistCredit>,
    #[serde(skip)]
    pub genres: Vec<Genre>,
    #[serde(skip)]
    pub release_group: ReleaseGroup,
    #[serde(skip)]
    pub media: Vec<Medium>,
}

