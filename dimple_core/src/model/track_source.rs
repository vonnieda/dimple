use dimple_core_macro::ModelSupport;

use crate::library::Library;

use super::{MediaFile, ModelBasics as _, Track};

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct TrackSource {
    pub key: Option<String>,
    pub track_key: Option<String>,
    pub blob_key: Option<String>,
    pub media_file_key: Option<String>,
}

impl TrackSource {
    pub fn track(&self, library: &Library) -> Option<Track> {
        self.track_key.clone().and_then(|key| Track::get(library, &key))
    }

    pub fn media_file(&self, library: &Library) -> Option<MediaFile> {
        self.media_file_key.clone().and_then(|key| MediaFile::get(library, &key))
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Blob, Track, TrackSource}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let blob = library.save(&Blob::default());
        let track = library.save(&Track::default());
        let model = library.save(&TrackSource {
            track_key: track.key.clone(),
            blob_key: blob.key.clone(),
            ..Default::default()
        });
        let model = library.save(&model);
        let model: TrackSource = library.get(&model.key.unwrap()).unwrap();
        assert!(model.blob_key == blob.key);
    }
}
