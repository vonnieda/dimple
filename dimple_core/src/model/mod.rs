
use std::{any::Any, time::{Duration, SystemTime}};

use chrono::{DateTime, Utc};
use dimage::DimageKind;
use rusqlite::{types::FromSql, Connection, Params, Row, ToSql};

mod artist;
pub use artist::Artist;

mod track;
pub use track::Track;

mod playlist;
pub use playlist::Playlist;

mod changelog;
pub use changelog::ChangeLog;

mod track_source;
pub use track_source::TrackSource;

mod media_file;
pub use media_file::MediaFile;

mod blob;
pub use blob::Blob;

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

mod dimage;
pub use dimage::Dimage;

mod dimage_ref;
pub use dimage_ref::DimageRef;

mod playlist_item;
pub use playlist_item::PlaylistItem;

use crate::library::Library;

pub trait FromRow {
    fn from_row(row: &Row) -> Self;
}

pub trait Diff {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> where Self: Sized;
    fn apply_diff(&mut self, diff: &[ChangeLog]);
}

pub trait Model: Sized + FromRow + Diff + Default + Clone + Send {
    fn type_name(&self) -> String;
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
    fn upsert(&self, conn: &Connection);
    fn log_changes(&self) -> bool;
    // fn as_any(&self) -> &dyn Any;
}

pub struct ChangeLogValue {
    pub val: Option<String>,
}

// TODO I think I can replace all of this with a generic over impl FromSql
// and ToSql
impl From<bool> for ChangeLogValue {
    fn from(value: bool) -> Self {
        ChangeLogValue {
            val: Some(if value { "true" } else { "false" }.to_string()),
        }
    }
}

impl From<ChangeLogValue> for bool {
    fn from(value: ChangeLogValue) -> Self {
        value.val.unwrap() == "true"
    }
}

impl From<Option<String>> for ChangeLogValue {
    fn from(value: Option<String>) -> Self {
        ChangeLogValue {
            val: value.clone(),
        }
    }
}

impl From<ChangeLogValue> for Option<String> {
    fn from(value: ChangeLogValue) -> Self {
        value.val
    }
}

impl From<String> for ChangeLogValue {
    fn from(value: String) -> Self {
        ChangeLogValue {
            val: Some(value)
        }
    }
}

impl From<ChangeLogValue> for String {
    fn from(value: ChangeLogValue) -> Self {
        value.val.unwrap()
    }
}

impl From<u64> for ChangeLogValue {
    fn from(value: u64) -> Self {
        ChangeLogValue {
            val: Some(value.to_string())
        }
    }
}

impl From<ChangeLogValue> for u64 {
    fn from(value: ChangeLogValue) -> Self {
        u64::from_str_radix(&value.val.unwrap(), 10).unwrap()
    }
}

impl From<ChangeLogValue> for Option<u64> {
    fn from(value: ChangeLogValue) -> Self {
        if let Some(value) = value.val {
            return Some(u64::from_str_radix(&value, 10).unwrap())
        }
        None
    }
}

impl From<Option<u64>> for ChangeLogValue {
    fn from(value: Option<u64>) -> Self {
        ChangeLogValue {
            val: value.map(|v| v.to_string())
        }
    }
}

impl From<u32> for ChangeLogValue {
    fn from(value: u32) -> Self {
        ChangeLogValue {
            val: Some(value.to_string())
        }
    }
}

impl From<ChangeLogValue> for u32 {
    fn from(value: ChangeLogValue) -> Self {
        u32::from_str_radix(&value.val.unwrap(), 10).unwrap()
    }
}

impl From<ChangeLogValue> for Option<u32> {
    fn from(value: ChangeLogValue) -> Self {
        if let Some(value) = value.val {
            return Some(u32::from_str_radix(&value, 10).unwrap())
        }
        None
    }
}

impl From<Option<u32>> for ChangeLogValue {
    fn from(value: Option<u32>) -> Self {
        ChangeLogValue {
            val: value.map(|v| v.to_string())
        }
    }
}

impl From<DateTime<Utc>> for ChangeLogValue {
    fn from(value: DateTime<Utc>) -> Self {
        ChangeLogValue {
            val: Some(value.to_rfc3339())
        }
    }
}

impl From<ChangeLogValue> for DateTime<Utc> {
    fn from(value: ChangeLogValue) -> Self {
        DateTime::parse_from_rfc3339(&value.val.unwrap()).unwrap().into()
    }
}

impl From<Vec<u8>> for ChangeLogValue {
    fn from(value: Vec<u8>) -> Self {
        todo!()
    }
}

impl From<ChangeLogValue> for Vec<u8> {
    fn from(value: ChangeLogValue) -> Self {
        todo!()
    }
}

pub trait ModelBasics<T> {
    fn get(library: &Library, key: &str) -> Option<T>;
    fn list(library: &Library) -> Vec<T>;
    fn save(&self, library: &Library) -> T;
    fn query(library: &Library, sql: &str, params: impl Params) -> Vec<T>;
}

impl <T: Model> ModelBasics<T> for T  {
    fn get(library: &Library, key: &str) -> Option<T> {
        library.get::<T>(key)
    }

    fn save(&self, library: &Library) -> T {
        library.save(self)
    }
    
    fn list(library: &Library) -> Vec<T> {
        library.list()
    }

    fn query(library: &Library, sql: &str, params: impl Params) -> Vec<T> {
        library.query(sql, params)
    }
}