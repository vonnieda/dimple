use dimple_core_nt_macro::ModelSupport;
use rusqlite::Row;

use super::{ChangeLog, Diff, FromRow, Model};

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub liked: bool,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:0557f771-4697-4d8d-807b-9576381b50b4?mode=memory&cache=shared");
        let mut model = library.save(&Track::default());
        assert!(model.key.is_some());
        assert!(model.artist.is_none());
        model.artist = Some("Artist".to_string());
        let model = library.save(&model);
        let model: Track = library.get(&model.key.unwrap()).unwrap();
        assert!(model.artist == Some("Artist".to_string()));
    }

    #[test]
    fn diff() {
        let a = Track::default();
        let b = Track {
            key: Some("key".to_string()),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
            liked: true,
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 5);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("artist".to_string()));
        assert!(diff[2].field == Some("album".to_string()));
        assert!(diff[3].field == Some("title".to_string()));
        assert!(diff[4].field == Some("liked".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Track::default();
        let b = Track {
            key: Some("key".to_string()),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
            liked: true,
        };
        let diff = a.diff(&b);
        let mut c = Track::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}