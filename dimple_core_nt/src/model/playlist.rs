use super::{Diff, FromRow, Track};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub tracks: Vec<Track>,
}

impl FromRow for Playlist {
    fn from_row(row: &rusqlite::Row) -> Self {
        todo!()
    }
}

impl Diff for Playlist {
    fn diff(&self, other: &Self) -> Vec<super::ChangeLog> where Self: Sized {
        todo!()
    }

    fn apply_diff(&mut self, diff: &[super::ChangeLog]) {
        todo!()
    }
}