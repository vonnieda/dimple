use std::hash::Hash;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::Artist;

// https://musicbrainz.org/doc/Artist_Credits
// > Artist credits can be added to tracks, recordings, releases, and release groups. 
// Note that this combines portions of the artist_credit_name table, too.
#[derive(Clone, Debug, Serialize, Deserialize, Default, ModelSupport)]
pub struct ArtistCredit {
    pub key: Option<String>,
    pub name: Option<String>,
    pub join_phrase: Option<String>,
    
    #[serde(skip)]
    pub artist: Artist,
}

