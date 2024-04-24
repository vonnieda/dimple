use std::any::Any;

use serde::Deserialize;
use serde::Serialize;

mod artist;
mod genre;
mod media_file;
mod medium;
mod playlist;
mod recording_source;
mod recording;
mod release_group;
mod release;
mod track;
mod known_id;
mod picture;

pub use artist::Artist;
pub use release_group::ReleaseGroup;
pub use release::Release;
pub use track::Track;
pub use medium::Medium;
pub use media_file::MediaFile;
pub use recording::Recording;
pub use recording_source::RecordingSource;
pub use genre::Genre;
pub use known_id::KnownId;
pub use picture::Picture;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Model {
    Artist(Artist),
    Genre(Genre),
    MediaFile(MediaFile),
    Medium(Medium),
    Recording(Recording),
    RecordingSource(RecordingSource),
    ReleaseGroup(ReleaseGroup),
    Release(Release),
    Track(Track),
    Picture(Picture),
}

pub trait Entity {
    fn key(&self) -> Option<String>;
    fn type_name(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn model(&self) -> Model;
}