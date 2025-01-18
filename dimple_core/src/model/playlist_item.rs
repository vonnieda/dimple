use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct PlaylistItem {
    pub key: Option<String>,
    pub playlist_key: String,
    pub ordinal: String,
    pub track_key: String,
}
