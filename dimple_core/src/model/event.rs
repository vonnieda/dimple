use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Event {
    pub key: Option<String>,
    pub timestamp: String,
    pub event_type: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub source_type: String,
    pub source: String,
}

