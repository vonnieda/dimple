use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::ArtistCredit;
use super::Genre;
use super::KnownIds;
use super::Recording;

// https://musicbrainz.org/doc/Track
// https://musicbrainz.org/ws/2/release/4d3ce256-ea71-44c5-8ce9-deb8f1e7dce4?inc=aliases%2Bartist-credits%2Blabels%2Bdiscids%2Brecordings&fmt=json
// > This entity is not visible to users on its own, only in the context of a
// release. It has an MBID, and contains a link to a recording, a title, 
// artist credit and position on its associated medium. 
// In the schema image it also has a medium (ref)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ModelSupport)]
pub struct Track {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: KnownIds,

    pub length: Option<u32>,
    // A text description of the position in the media, such as A1
    pub number: Option<u32>,
    // 1 ased ordinal within the media
    pub position: Option<u32>,

    #[serde(skip)]
    pub recording: Recording,
    #[serde(skip)]
    pub genres: Vec<Genre>,
    #[serde(skip)]
    pub artist_credits: Vec<ArtistCredit>,
}
