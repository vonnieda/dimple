use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: Option<String>,

    pub file_path: String,

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
        let model = library.save(&MediaFile::default()).unwrap();
        assert!(model.id.is_some());
    }
}