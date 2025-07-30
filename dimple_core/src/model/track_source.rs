use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{MediaFile, ModelBasics as _, Track};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackSource {
    pub id: Option<String>,
    pub track_id: Option<String>,
    pub blob_id: Option<String>,
    pub media_file_id: Option<String>,
}

impl TrackSource {
    pub fn track(&self, library: &Library) -> Option<Track> {
        self.track_id.clone().and_then(|id| library.get(&id))
    }

    pub fn media_file(&self, library: &Library) -> Option<MediaFile> {
        self.media_file_id.clone().and_then(|id| library.get(&id))
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
            track_id: track.id.clone(),
            blob_id: blob.id.clone(),
            ..Default::default()
        });
        let model = library.save(&model);
        let model: TrackSource = library.get(&model.id.unwrap()).unwrap();
        assert!(model.blob_id == blob.id);
    }
}
