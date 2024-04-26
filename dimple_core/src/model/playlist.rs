use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
}
