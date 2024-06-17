use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ModelSupport)]
pub struct Blob {
    pub key: Option<String>,
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }
}
