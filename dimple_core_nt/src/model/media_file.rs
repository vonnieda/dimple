use dimple_core_nt_macro::ModelSupport;
use super::{ChangeLog, Diff, FromRow, Model};
use rusqlite::Row;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct MediaFile {
    pub key: Option<String>,

    pub file_path: String,
    pub sha256: String,

    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::Diff};

    use super::MediaFile;

    #[test]
    fn library_crud() {
        let library = Library::open("file:e57e4eb6-c6af-43d2-ac26-d1edf8972951?mode=memory&cache=shared");
        let mut model = library.save(&MediaFile::default());
        assert!(model.key.is_some());
        assert!(model.artist.is_none());
        model.artist = Some("Artist".to_string());
        let model = library.save(&model);
        let model: MediaFile = library.get(&model.key.unwrap()).unwrap();
        assert!(model.artist == Some("Artist".to_string()));
    }

    #[test]
    fn diff() {
        let a = MediaFile::default();
        let b = MediaFile {
            key: Some("key".to_string()),
            file_path: "file_path".to_string(),
            sha256: "sha256".to_string(),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 6);
        assert!(diff[0].model == "MediaFile".to_string());
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("file_path".to_string()));
        assert!(diff[2].field == Some("sha256".to_string()));
        assert!(diff[3].field == Some("artist".to_string()));
        assert!(diff[4].field == Some("album".to_string()));
        assert!(diff[5].field == Some("title".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = MediaFile::default();
        let b = MediaFile {
            key: Some("key".to_string()),
            file_path: "file_path".to_string(),
            sha256: "sha256".to_string(),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
        };
        let diff = a.diff(&b);
        let mut c = MediaFile::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}