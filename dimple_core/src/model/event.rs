use dimple_core_macro::ModelSupport;

// TODO rethinking calling this Event because I'll eventually want to track
// concert type events. Maybe this is History, or Interaction, or Listen, or Scrobble?

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

