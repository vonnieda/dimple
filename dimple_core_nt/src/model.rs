use rusqlite::{Connection, Row};

mod track;
pub use track::Track;

mod playlist;
pub use playlist::Playlist;

mod changelog;
pub use changelog::ChangeLog;

mod track_source;
// pub use track_source::TrackSource;

mod media_file;
pub use media_file::MediaFile;

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
}
