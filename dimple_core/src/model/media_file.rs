use chrono::{DateTime, Utc};
use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct MediaFile {
    pub key: Option<String>,

    pub file_path: String,
    pub sha256: String,

    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,    
    pub genre: Option<String>,

    // // TODO Duration, I think, and probably ns vs ms.
    pub length_ms: Option<u64>,
    // // pub media_position: Option<u32>,

    pub lyrics: Option<String>,
    // // TODO LRC format (https://en.wikipedia.org/wiki/LRC_(file_format)) for
    // // now, convert to a model later.
    pub synced_lyrics: Option<String>,

    /// A multi-value tag containing the MBIDs for the track artists.
    pub musicbrainz_artist_id: Option<String>,
    pub musicbrainz_release_group_id: Option<String>,
    pub musicbrainz_album_id: Option<String>,
    /// A multi-value tag containing the MBIDs for the release artists.
    pub musicbrainz_album_artist_id: Option<String>,
    pub musicbrainz_track_id: Option<String>,
    pub musicbrainz_recording_id: Option<String>,
    pub musicbrainz_release_track_id: Option<String>,
    pub musicbrainz_genre_id: Option<String>,

    pub last_modified: DateTime<Utc>,
    pub last_imported: DateTime<Utc>,
}

// fn asdasd() {
//     let mf = MediaFile::default();
//     let library = Library::open_temporary();
//     let params = params!
// }

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
            ..Default::default()
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
            ..Default::default()
        };
        let diff = a.diff(&b);
        let mut c = MediaFile::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}