use dimple_core_nt_macro::ModelSupport;
use super::{ChangeLog, Diff, FromRow, Model};
use rusqlite::Row;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
}

