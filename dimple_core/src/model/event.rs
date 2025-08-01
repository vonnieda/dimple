use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// TODO rethinking calling this Event because I'll eventually want to track
// concert type events. Maybe this is History, or Interaction, or Listen, or Scrobble?

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub source_type: String,
    pub source: String,
}

