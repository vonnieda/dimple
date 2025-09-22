use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scrobble {
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub scrobble_type: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub source_type: String,
    pub source: String,
}

