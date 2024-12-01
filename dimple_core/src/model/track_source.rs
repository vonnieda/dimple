use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct TrackSource {
    pub key: Option<String>,
    pub track_key: String,
    pub blob_key: String,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Blob, Diff, Track, TrackSource}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:712f9444-5755-4795-a75f-a4c33fd695c6?mode=memory&cache=shared");
        let blob = library.save(&Blob::default());
        let track = library.save(&Track::default());
        let model = library.save(&TrackSource {
            track_key: track.key.clone().unwrap(),
            blob_key: blob.key.clone().unwrap(),
            ..Default::default()
        });
        let model = library.save(&model);
        let model: TrackSource = library.get(&model.key.unwrap()).unwrap();
        assert!(model.blob_key == blob.key.unwrap());
    }

    #[test]
    fn diff() {
        let a = TrackSource::default();
        let b = TrackSource {
            key: Some("key".to_string()),
            track_key: "track_key".to_string(),
            blob_key: "blob_key".to_string(),
            ..Default::default()
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 3);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("track_key".to_string()));
        assert!(diff[2].field == Some("blob_key".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = TrackSource::default();
        let b = TrackSource {
            key: Some("key".to_string()),
            track_key: "track_key".to_string(),
            blob_key: "blob_key".to_string(),
            ..Default::default()
        };
        let diff = a.diff(&b);
        let mut c = TrackSource::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}