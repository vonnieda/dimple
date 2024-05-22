use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

// https://musicbrainz.org/doc/Artist_Credits
// Artist credits can be added to tracks, recordings, releases, and release groups. 
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct ArtistCredit {
    pub key: Option<String>,
    pub name: Option<String>,
    pub join_phrase: Option<String>,
}
