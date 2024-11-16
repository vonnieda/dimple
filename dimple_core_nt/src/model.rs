use rusqlite::{Connection, Row};

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

use crate::library::Library;

pub trait FromRow {
    fn from_row(row: &Row) -> Self;
}

pub trait Diff {
    fn diff(&self, other: &Self) -> Vec<ChangeLog> where Self: Sized;
    fn apply_diff(&mut self, diff: &[ChangeLog]);
}

pub trait Model: Sized + FromRow + Diff + Default + Clone {
    fn table_name() -> String;
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
    fn upsert(&self, conn: &Connection);
    fn log_changes() -> bool;
    fn hydrate(&mut self, library: &Library) {}
}

struct ChangeLogValue {
    pub val: Option<String>,
}

impl From<bool> for ChangeLogValue {
    fn from(value: bool) -> Self {
        ChangeLogValue {
            val: Some(if value { "true" } else { "false" }.to_string()),
        }
    }
}

impl From<Option<String>> for ChangeLogValue {
    fn from(value: Option<String>) -> Self {
        ChangeLogValue {
            val: value.clone(),
        }
    }
}

impl From<String> for ChangeLogValue {
    fn from(value: String) -> Self {
        todo!()
    }
}

impl From<ChangeLogValue> for Option<String> {
    fn from(value: ChangeLogValue) -> Self {
        value.val
    }
}

impl From<ChangeLogValue> for bool {
    fn from(value: ChangeLogValue) -> Self {
        value.val.unwrap() == "true"
    }
}

impl From<ChangeLogValue> for String {
    fn from(value: ChangeLogValue) -> Self {
        todo!()
    }
}