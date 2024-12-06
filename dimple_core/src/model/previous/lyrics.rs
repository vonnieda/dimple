use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ModelSupport)]
pub struct Lyrics {
    pub key: Option<String>,

    pub lyrics: String,
    pub synchronized_lyrics: Vec<(String, u32)>,
}
