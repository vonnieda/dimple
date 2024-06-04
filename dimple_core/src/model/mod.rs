use std::any::Any;

use serde::Deserialize;
use serde::Serialize;

mod artist;
mod artist_credit;
mod blob;
mod genre;
mod medium;
mod playlist;
mod recording_source;
mod recording;
mod release_group;
mod release;
mod track;
mod known_id;
mod picture;
mod playlist_item;

pub use artist::Artist;
pub use artist_credit::ArtistCredit;
pub use blob::Blob;
pub use release_group::ReleaseGroup;
pub use release::Release;
pub use track::Track;
pub use medium::Medium;
pub use recording::Recording;
pub use recording_source::RecordingSource;
pub use genre::Genre;
pub use known_id::KnownIds;
pub use picture::Picture;
pub use playlist::Playlist;
pub use playlist_item::PlaylistItem;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Model {
    Artist(Artist),
    ArtistCredit(ArtistCredit),
    Blob(Blob),
    Genre(Genre),
    Medium(Medium),
    Recording(Recording),
    RecordingSource(RecordingSource),
    ReleaseGroup(ReleaseGroup),
    Release(Release),
    Track(Track),
    Picture(Picture),
    Playlist(Playlist),
    PlaylistItem(PlaylistItem),
}

impl Model {
    pub fn entity(&self) -> &dyn Entity {
        match self {
            Model::Playlist(v) => v,
            Model::Artist(v) => v,
            Model::ArtistCredit(v) => v,
            Model::Blob(v) => v,
            Model::Genre(v) => v,
            Model::Medium(v) => v,
            Model::Recording(v) => v,
            Model::RecordingSource(v) => v,
            Model::ReleaseGroup(v) => v,
            Model::Release(v) => v,
            Model::Track(v) => v,
            Model::Picture(v) => v,
            Model::PlaylistItem(v) => v,
        }
    }

    pub fn set_key(&mut self, key: Option<String>) {
        match self {
            Model::Playlist(v) => v.key = key,
            Model::Artist(v) => v.key = key,
            Model::ArtistCredit(v) => v.key = key,
            Model::Blob(v) => v.key = key,
            Model::Genre(v) => v.key = key,
            Model::Medium(v) => v.key = key,
            Model::Recording(v) => v.key = key,
            Model::RecordingSource(v) => v.key = key,
            Model::ReleaseGroup(v) => v.key = key,
            Model::Release(v) => v.key = key,
            Model::Track(v) => v.key = key,
            Model::Picture(v) => v.key = key,
            Model::PlaylistItem(v) => v.key = key,
        }
    }
}

pub trait Entity {
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
    fn type_name(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn model(&self) -> Model;
}