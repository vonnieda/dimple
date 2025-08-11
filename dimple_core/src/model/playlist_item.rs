use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: Option<String>,
    pub playlist_id: String,
    pub ordinal: String,
    pub track_id: String,
    
    pub deleted: bool,
}
