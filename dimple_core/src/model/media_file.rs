use std::time::Duration;

use chrono::{DateTime, Utc};
use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct MediaFile {
    pub key: Option<String>,

    pub file_path: String,
    // TODO I think I'm going to remove this and only use sha256 for sync
    // It's just too slow to have to worry about for every import.
    pub sha256: String,

    pub last_modified: DateTime<Utc>,
    pub last_imported: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library};

    use super::MediaFile;

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let model = library.save(&MediaFile::default());
        assert!(model.key.is_some());
    }
}