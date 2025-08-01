
use std::{any::Any, time::{Duration, SystemTime}};

use chrono::{DateTime, Utc};
use dimage::DimageKind;

mod artist;
pub use artist::Artist;

mod track;
use dimple_db::{db::Entity, rusqlite::Params};
pub use track::Track;

mod playlist;
pub use playlist::Playlist;

mod track_source;
pub use track_source::TrackSource;

mod media_file;
pub use media_file::MediaFile;

mod genre;
pub use genre::Genre;

mod release;
pub use release::Release;

mod event;
pub use event::Event;

mod artist_ref;
pub use artist_ref::ArtistRef;

mod genre_ref;
pub use genre_ref::GenreRef;

mod link;
pub use link::Link;

mod link_ref;
pub use link_ref::LinkRef;

pub mod dimage;
pub use dimage::Dimage;

mod dimage_ref;
pub use dimage_ref::DimageRef;

mod playlist_item;
pub use playlist_item::PlaylistItem;

use crate::library::Library;

// TODO rename EntityBasics, or maybe get rid of. library interface is fine
// and it's annoying doing it two ways
pub trait ModelBasics<T> {
    fn get(library: &Library, key: &str) -> Option<T>;
    fn list(library: &Library) -> Vec<T>;
    fn save(&self, library: &Library) -> T;
    fn query(library: &Library, sql: &str, params: impl Params) -> Vec<T>;
    fn find(library: &Library, sql: &str, params: impl Params) -> Option<T>;
}

impl <T: Entity> ModelBasics<T> for T {
    fn get(library: &Library, key: &str) -> Option<T> {
        library.get(key)
    }

    fn list(library: &Library) -> Vec<T> {
        library.list()
    }

    fn save(&self, library: &Library) -> T {
        library.save(self).unwrap()
    }
    
    fn query(library: &Library, sql: &str, params: impl Params) -> Vec<T> {
        library.query(sql, params)
    }
    
    fn find(library: &Library, sql: &str, params: impl Params) -> Option<T> {
        library.find(sql, params)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DimpleEntity {
    Artist(Artist),
    Track(Track),
    Genre(Genre),
    Release(Release),
    Playlist(Playlist),
}

impl DimpleEntity {
    pub fn id(&self) -> String {
        match self {
            DimpleEntity::Artist(a) => a.id.clone().unwrap(),
            DimpleEntity::Track(t) => t.id.clone().unwrap(),
            DimpleEntity::Genre(g) => g.id.clone().unwrap(),
            DimpleEntity::Release(r) => r.id.clone().unwrap(),
            DimpleEntity::Playlist(playlist) => playlist.id.clone().unwrap(),
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            DimpleEntity::Artist(a) => "Artist".to_string(),
            DimpleEntity::Track(t) => "Track".to_string(),
            DimpleEntity::Genre(t) => "Genre".to_string(),
            DimpleEntity::Release(t) => "Release".to_string(),
            DimpleEntity::Playlist(t) => "Playlist".to_string(),
        }
    }
}

macro_rules! impl_from_for_dimple_entity {
      ($($type:ty => $variant:ident),+) => {
          $(
              impl From<&$type> for DimpleEntity {
                  fn from(value: &$type) -> Self {
                      DimpleEntity::$variant(value.clone())
                  }
              }

              impl From<$type> for DimpleEntity {
                  fn from(value: $type) -> Self {
                      DimpleEntity::$variant(value)
                  }
              }
          )+
      };
  }

impl_from_for_dimple_entity! {
    Artist => Artist,
    Track => Track,
    Genre => Genre,
    Release => Release,
    Playlist => Playlist
}

