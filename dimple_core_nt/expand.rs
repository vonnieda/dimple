#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod model {
    use rusqlite::{Connection, Row};
    mod track {
        use dimple_core_nt_macro::ModelSupport;
        use rusqlite::Row;
        use super::{ChangeLog, Diff, FromRow, Model};
        pub struct Track {
            pub key: Option<String>,
            pub artist: Option<String>,
            pub album: Option<String>,
            pub title: Option<String>,
            pub liked: bool,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Track {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "Track",
                    "key",
                    &self.key,
                    "artist",
                    &self.artist,
                    "album",
                    &self.album,
                    "title",
                    &self.title,
                    "liked",
                    &&self.liked,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Track {
            #[inline]
            fn clone(&self) -> Track {
                Track {
                    key: ::core::clone::Clone::clone(&self.key),
                    artist: ::core::clone::Clone::clone(&self.artist),
                    album: ::core::clone::Clone::clone(&self.album),
                    title: ::core::clone::Clone::clone(&self.title),
                    liked: ::core::clone::Clone::clone(&self.liked),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Track {
            #[inline]
            fn default() -> Track {
                Track {
                    key: ::core::default::Default::default(),
                    artist: ::core::default::Default::default(),
                    album: ::core::default::Default::default(),
                    title: ::core::default::Default::default(),
                    liked: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Track {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Track {
            #[inline]
            fn eq(&self, other: &Track) -> bool {
                self.key == other.key && self.artist == other.artist
                    && self.album == other.album && self.title == other.title
                    && self.liked == other.liked
            }
        }
        impl FromRow for Track {
            fn from_row(row: &Row) -> Self {
                Self {}
            }
        }
        impl Model for Track {
            fn table_name() -> String {
                "Track".to_string()
            }
            fn key(&self) -> Option<String> {
                self.key.clone()
            }
            fn upsert(&self, conn: &rusqlite::Connection) {
                conn.execute(
                        "INSERT OR REPLACE INTO Track 
                                    (key, artist, album, title, liked) 
                                    VALUES (?1, ?2, ?3, ?4, ?5)",
                        (&self.key, &self.artist, &self.album, &self.title, &self.liked),
                    )
                    .unwrap();
            }
            fn set_key(&mut self, key: Option<String>) {
                self.key = key.clone();
            }
            fn log_changes() -> bool {
                true
            }
        }
        impl Diff for Track {
            fn diff(&self, other: &Self) -> Vec<ChangeLog> {
                let mut diff = ::alloc::vec::Vec::new();
                if self.key != other.key {
                    diff.push(ChangeLog {
                        model: "Track".to_string(),
                        op: "set".to_string(),
                        field: Some("key".to_string()),
                        ..Default::default()
                    });
                }
                if self.artist != other.artist {
                    diff.push(ChangeLog {
                        model: "Track".to_string(),
                        op: "set".to_string(),
                        field: Some("artist".to_string()),
                        ..Default::default()
                    });
                }
                if self.album != other.album {
                    diff.push(ChangeLog {
                        model: "Track".to_string(),
                        op: "set".to_string(),
                        field: Some("album".to_string()),
                        ..Default::default()
                    });
                }
                if self.title != other.title {
                    diff.push(ChangeLog {
                        model: "Track".to_string(),
                        op: "set".to_string(),
                        field: Some("title".to_string()),
                        ..Default::default()
                    });
                }
                if self.liked != other.liked {
                    diff.push(ChangeLog {
                        model: "Track".to_string(),
                        op: "set".to_string(),
                        field: Some("liked".to_string()),
                        ..Default::default()
                    });
                }
                diff
            }
            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                for change in diff {
                    if change.op == "set" {
                        if let Some(field) = change.field.clone() {}
                    }
                }
            }
        }
    }
    pub use track::Track;
    mod playlist {
        use rusqlite::Row;
        use crate::library::Library;
        use super::{ChangeLog, Diff, FromRow, Model, Track};
        pub struct Playlist {
            pub key: Option<String>,
            pub name: Option<String>,
            pub tracks: Vec<Track>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Playlist {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "Playlist",
                    "key",
                    &self.key,
                    "name",
                    &self.name,
                    "tracks",
                    &&self.tracks,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Playlist {
            #[inline]
            fn clone(&self) -> Playlist {
                Playlist {
                    key: ::core::clone::Clone::clone(&self.key),
                    name: ::core::clone::Clone::clone(&self.name),
                    tracks: ::core::clone::Clone::clone(&self.tracks),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Playlist {
            #[inline]
            fn default() -> Playlist {
                Playlist {
                    key: ::core::default::Default::default(),
                    name: ::core::default::Default::default(),
                    tracks: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Playlist {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Playlist {
            #[inline]
            fn eq(&self, other: &Playlist) -> bool {
                self.key == other.key && self.name == other.name
                    && self.tracks == other.tracks
            }
        }
        impl FromRow for Playlist {
            fn from_row(row: &Row) -> Self {
                Self {
                    key: row.get("key").unwrap(),
                    name: row.get("name").unwrap(),
                    ..Default::default()
                }
            }
        }
        impl Diff for Playlist {
            fn diff(&self, other: &Self) -> Vec<ChangeLog> {
                let mut diff = ::alloc::vec::Vec::new();
                if self.key != other.key {
                    diff.push(ChangeLog {
                        model: "Playlist".to_string(),
                        op: "set".to_string(),
                        field: Some("key".to_string()),
                        value: other.key.clone(),
                        ..Default::default()
                    });
                }
                if self.name != other.name {
                    diff.push(ChangeLog {
                        model: "Playlist".to_string(),
                        op: "set".to_string(),
                        field: Some("name".to_string()),
                        value: other.name.clone(),
                        ..Default::default()
                    });
                }
                diff
            }
            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                for change in diff {
                    if change.op == "set" {
                        if let Some(field) = change.field.clone() {
                            if &field == "key" {
                                self.key = change.value.clone();
                            }
                            if &field == "name" {
                                self.name = change.value.clone();
                            }
                        }
                    }
                }
            }
        }
        impl Model for Playlist {
            fn table_name() -> String {
                "Playlist".to_string()
            }
            fn key(&self) -> Option<String> {
                self.key.clone()
            }
            fn upsert(&self, conn: &rusqlite::Connection) {
                conn.execute(
                        "INSERT OR REPLACE INTO Playlist 
            (key, name) 
            VALUES (?1, ?2)",
                        (&self.key, &self.name),
                    )
                    .unwrap();
            }
            fn set_key(&mut self, key: Option<String>) {
                self.key = key.clone();
            }
            fn log_changes() -> bool {
                true
            }
            fn hydrate(&mut self, library: &Library) {
                let conn = library.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT
            Track.*
            FROM PlaylistItem
            JOIN Track ON (Track.key = PlaylistItem.Track_key)
            WHERE PlaylistItem.playlist_key = ?1",
                    )
                    .unwrap();
                let mut rows = stmt.query((self.key.clone(),)).unwrap();
                while let Some(row) = rows.next().unwrap() {
                    self.tracks.push(Track::from_row(row));
                }
            }
        }
    }
    pub use playlist::Playlist;
    mod changelog {
        use rusqlite::Row;
        use super::{Diff, FromRow, Model};
        pub struct ChangeLog {
            pub key: Option<String>,
            pub actor: String,
            pub timestamp: String,
            pub model: String,
            pub model_key: String,
            pub op: String,
            pub field: Option<String>,
            pub value: Option<String>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ChangeLog {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "key",
                    "actor",
                    "timestamp",
                    "model",
                    "model_key",
                    "op",
                    "field",
                    "value",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.key,
                    &self.actor,
                    &self.timestamp,
                    &self.model,
                    &self.model_key,
                    &self.op,
                    &self.field,
                    &&self.value,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "ChangeLog",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ChangeLog {
            #[inline]
            fn clone(&self) -> ChangeLog {
                ChangeLog {
                    key: ::core::clone::Clone::clone(&self.key),
                    actor: ::core::clone::Clone::clone(&self.actor),
                    timestamp: ::core::clone::Clone::clone(&self.timestamp),
                    model: ::core::clone::Clone::clone(&self.model),
                    model_key: ::core::clone::Clone::clone(&self.model_key),
                    op: ::core::clone::Clone::clone(&self.op),
                    field: ::core::clone::Clone::clone(&self.field),
                    value: ::core::clone::Clone::clone(&self.value),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for ChangeLog {
            #[inline]
            fn default() -> ChangeLog {
                ChangeLog {
                    key: ::core::default::Default::default(),
                    actor: ::core::default::Default::default(),
                    timestamp: ::core::default::Default::default(),
                    model: ::core::default::Default::default(),
                    model_key: ::core::default::Default::default(),
                    op: ::core::default::Default::default(),
                    field: ::core::default::Default::default(),
                    value: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ChangeLog {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ChangeLog {
            #[inline]
            fn eq(&self, other: &ChangeLog) -> bool {
                self.key == other.key && self.actor == other.actor
                    && self.timestamp == other.timestamp && self.model == other.model
                    && self.model_key == other.model_key && self.op == other.op
                    && self.field == other.field && self.value == other.value
            }
        }
        impl FromRow for ChangeLog {
            fn from_row(row: &Row) -> Self {
                Self {
                    key: row.get("key").unwrap(),
                    actor: row.get("actor").unwrap(),
                    timestamp: row.get("timestamp").unwrap(),
                    model: row.get("model").unwrap(),
                    model_key: row.get("model_key").unwrap(),
                    op: row.get("op").unwrap(),
                    field: row.get("field").unwrap(),
                    value: row.get("value").unwrap(),
                }
            }
        }
        impl Diff for ChangeLog {
            fn diff(&self, other: &Self) -> Vec<ChangeLog>
            where
                Self: Sized,
            {
                ::core::panicking::panic("not yet implemented")
            }
            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                ::core::panicking::panic("not yet implemented")
            }
        }
        impl Model for ChangeLog {
            fn table_name() -> String {
                "ChangeLog".to_string()
            }
            fn key(&self) -> Option<String> {
                self.key.clone()
            }
            fn upsert(&self, conn: &rusqlite::Connection) {
                conn.execute(
                        "INSERT OR REPLACE INTO ChangeLog 
            (key, actor, timestamp, model, model_key, op, field, value) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        (
                            &self.key,
                            &self.actor,
                            &self.timestamp,
                            &self.model,
                            &self.model_key,
                            &self.op,
                            &self.field,
                            &self.value,
                        ),
                    )
                    .unwrap();
            }
            fn set_key(&mut self, key: Option<String>) {
                self.key = key.clone();
            }
            fn log_changes() -> bool {
                false
            }
        }
    }
    pub use changelog::ChangeLog;
    mod track_source {
        use dimple_core_nt_macro::ModelSupport;
        use rusqlite::Row;
        use super::{ChangeLog, Diff, FromRow, Model};
        pub struct TrackSource {
            pub key: Option<String>,
            pub track_key: String,
            pub blob_key: Option<String>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TrackSource {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "TrackSource",
                    "key",
                    &self.key,
                    "track_key",
                    &self.track_key,
                    "blob_key",
                    &&self.blob_key,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TrackSource {
            #[inline]
            fn clone(&self) -> TrackSource {
                TrackSource {
                    key: ::core::clone::Clone::clone(&self.key),
                    track_key: ::core::clone::Clone::clone(&self.track_key),
                    blob_key: ::core::clone::Clone::clone(&self.blob_key),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for TrackSource {
            #[inline]
            fn default() -> TrackSource {
                TrackSource {
                    key: ::core::default::Default::default(),
                    track_key: ::core::default::Default::default(),
                    blob_key: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TrackSource {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TrackSource {
            #[inline]
            fn eq(&self, other: &TrackSource) -> bool {
                self.key == other.key && self.track_key == other.track_key
                    && self.blob_key == other.blob_key
            }
        }
        impl FromRow for TrackSource {
            fn from_row(row: &Row) -> Self {
                Self {
                    key: row.get("key").unwrap(),
                    track_key: row.get("track_key").unwrap(),
                    blob_key: row.get("blob_key").unwrap(),
                }
            }
        }
        impl Diff for TrackSource {
            fn diff(&self, other: &Self) -> Vec<ChangeLog> {
                let mut diff = ::alloc::vec::Vec::new();
                if self.key != other.key {
                    diff.push(ChangeLog {
                        model: "TrackSource".to_string(),
                        op: "set".to_string(),
                        field: Some("key".to_string()),
                        value: other.key.clone(),
                        ..Default::default()
                    });
                }
                if self.track_key != other.track_key {
                    diff.push(ChangeLog {
                        model: "TrackSource".to_string(),
                        op: "set".to_string(),
                        field: Some("track_key".to_string()),
                        value: Some(other.track_key.clone()),
                        ..Default::default()
                    });
                }
                if self.blob_key != other.blob_key {
                    diff.push(ChangeLog {
                        model: "TrackSource".to_string(),
                        op: "set".to_string(),
                        field: Some("blob_key".to_string()),
                        value: other.blob_key.clone(),
                        ..Default::default()
                    });
                }
                diff
            }
            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                for change in diff {
                    if change.op == "set" {
                        if let Some(field) = change.field.clone() {
                            if &field == "key" {
                                self.key = change.value.clone();
                            }
                            if &field == "track_key" {
                                self.track_key = change.value.clone().unwrap();
                            }
                            if &field == "blob_key" {
                                self.blob_key = change.value.clone();
                            }
                        }
                    }
                }
            }
        }
        impl Model for TrackSource {
            fn table_name() -> String {
                "TrackSource".to_string()
            }
            fn key(&self) -> Option<String> {
                self.key.clone()
            }
            fn set_key(&mut self, key: Option<String>) {
                self.key = key.clone();
            }
            fn upsert(&self, conn: &rusqlite::Connection) {
                conn.execute(
                        "INSERT OR REPLACE INTO TrackSource 
            (key, track_key, blob_key) 
            VALUES (?1, ?2, ?3)",
                        (&self.key, &self.track_key, &self.blob_key),
                    )
                    .unwrap();
            }
            fn log_changes() -> bool {
                true
            }
        }
    }
    pub use track_source::TrackSource;
    mod media_file {
        use rusqlite::Row;
        use super::{ChangeLog, Diff, FromRow, Model};
        pub struct MediaFile {
            pub key: Option<String>,
            pub file_path: String,
            pub sha256: String,
            pub artist: Option<String>,
            pub album: Option<String>,
            pub title: Option<String>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for MediaFile {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "key",
                    "file_path",
                    "sha256",
                    "artist",
                    "album",
                    "title",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.key,
                    &self.file_path,
                    &self.sha256,
                    &self.artist,
                    &self.album,
                    &&self.title,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "MediaFile",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for MediaFile {
            #[inline]
            fn clone(&self) -> MediaFile {
                MediaFile {
                    key: ::core::clone::Clone::clone(&self.key),
                    file_path: ::core::clone::Clone::clone(&self.file_path),
                    sha256: ::core::clone::Clone::clone(&self.sha256),
                    artist: ::core::clone::Clone::clone(&self.artist),
                    album: ::core::clone::Clone::clone(&self.album),
                    title: ::core::clone::Clone::clone(&self.title),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for MediaFile {
            #[inline]
            fn default() -> MediaFile {
                MediaFile {
                    key: ::core::default::Default::default(),
                    file_path: ::core::default::Default::default(),
                    sha256: ::core::default::Default::default(),
                    artist: ::core::default::Default::default(),
                    album: ::core::default::Default::default(),
                    title: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for MediaFile {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for MediaFile {
            #[inline]
            fn eq(&self, other: &MediaFile) -> bool {
                self.key == other.key && self.file_path == other.file_path
                    && self.sha256 == other.sha256 && self.artist == other.artist
                    && self.album == other.album && self.title == other.title
            }
        }
        impl FromRow for MediaFile {
            fn from_row(row: &Row) -> Self {
                Self {
                    key: row.get("key").unwrap(),
                    file_path: row.get("file_path").unwrap(),
                    sha256: row.get("sha256").unwrap(),
                    artist: row.get("artist").unwrap(),
                    album: row.get("album").unwrap(),
                    title: row.get("title").unwrap(),
                }
            }
        }
        impl Diff for MediaFile {
            fn diff(&self, other: &Self) -> Vec<ChangeLog> {
                let mut diff = ::alloc::vec::Vec::new();
                if self.key != other.key {
                    diff.push(ChangeLog {
                        model: "MediaFile".to_string(),
                        op: "set".to_string(),
                        field: Some("key".to_string()),
                        value: other.key.clone(),
                        ..Default::default()
                    });
                }
                if self.file_path != other.file_path {
                    diff.push(ChangeLog {
                        model: "MediaFile".to_string(),
                        op: "set".to_string(),
                        field: Some("file_path".to_string()),
                        value: Some(other.file_path.clone()),
                        ..Default::default()
                    });
                }
                if self.sha256 != other.sha256 {
                    diff.push(ChangeLog {
                        model: "MediaFile".to_string(),
                        op: "set".to_string(),
                        field: Some("sha256".to_string()),
                        value: Some(other.sha256.clone()),
                        ..Default::default()
                    });
                }
                if self.artist != other.artist {
                    diff.push(ChangeLog {
                        model: "MediaFile".to_string(),
                        op: "set".to_string(),
                        field: Some("artist".to_string()),
                        value: other.artist.clone(),
                        ..Default::default()
                    });
                }
                if self.album != other.album {
                    diff.push(ChangeLog {
                        model: "MediaFile".to_string(),
                        op: "set".to_string(),
                        field: Some("album".to_string()),
                        value: other.album.clone(),
                        ..Default::default()
                    });
                }
                if self.title != other.title {
                    diff.push(ChangeLog {
                        model: "MediaFile".to_string(),
                        op: "set".to_string(),
                        field: Some("title".to_string()),
                        value: other.title.clone(),
                        ..Default::default()
                    });
                }
                diff
            }
            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                for change in diff {
                    if change.op == "set" {
                        if let Some(field) = change.field.clone() {
                            if &field == "key" {
                                self.key = change.value.clone();
                            }
                            if &field == "file_path" {
                                self.file_path = change.value.clone().unwrap();
                            }
                            if &field == "sha256" {
                                self.sha256 = change.value.clone().unwrap();
                            }
                            if &field == "artist" {
                                self.artist = change.value.clone();
                            }
                            if &field == "album" {
                                self.album = change.value.clone();
                            }
                            if &field == "title" {
                                self.title = change.value.clone();
                            }
                        }
                    }
                }
            }
        }
        impl Model for MediaFile {
            fn table_name() -> String {
                "MediaFile".to_string()
            }
            fn key(&self) -> Option<String> {
                self.key.clone()
            }
            fn upsert(&self, conn: &rusqlite::Connection) {
                conn.execute(
                        "INSERT OR REPLACE INTO MediaFile 
            (key, artist, album, title, file_path, sha256) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        (
                            &self.key,
                            &self.artist,
                            &self.album,
                            &self.title,
                            &self.file_path,
                            &self.sha256,
                        ),
                    )
                    .unwrap();
            }
            fn set_key(&mut self, key: Option<String>) {
                self.key = key.clone();
            }
            fn log_changes() -> bool {
                true
            }
        }
    }
    pub use media_file::MediaFile;
    mod blob {
        use rusqlite::Row;
        use sha2::{Sha256, Digest};
        use symphonia::core::checksum::Md5;
        use super::{ChangeLog, Diff, FromRow, Model};
        pub struct Blob {
            pub key: Option<String>,
            pub sha256: String,
            pub length: u64,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Blob {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "Blob",
                    "key",
                    &self.key,
                    "sha256",
                    &self.sha256,
                    "length",
                    &&self.length,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Blob {
            #[inline]
            fn clone(&self) -> Blob {
                Blob {
                    key: ::core::clone::Clone::clone(&self.key),
                    sha256: ::core::clone::Clone::clone(&self.sha256),
                    length: ::core::clone::Clone::clone(&self.length),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Blob {
            #[inline]
            fn default() -> Blob {
                Blob {
                    key: ::core::default::Default::default(),
                    sha256: ::core::default::Default::default(),
                    length: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Blob {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Blob {
            #[inline]
            fn eq(&self, other: &Blob) -> bool {
                self.key == other.key && self.sha256 == other.sha256
                    && self.length == other.length
            }
        }
        impl Blob {
            pub fn read(path: &str) -> Self {
                let path = std::fs::canonicalize(path).unwrap();
                let content = std::fs::read(&path).unwrap();
                let sha256 = Self::calculate_sha256(&content);
                Self {
                    key: None,
                    sha256: sha256,
                    length: content.len() as u64,
                }
            }
            fn calculate_sha256(data: &Vec<u8>) -> String {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let result = hasher.finalize();
                {
                    let res = ::alloc::fmt::format(format_args!("{0:x}", result));
                    res
                }
            }
        }
        impl FromRow for Blob {
            fn from_row(row: &Row) -> Self {
                Self {
                    key: row.get("key").unwrap(),
                    sha256: row.get("sha256").unwrap(),
                    length: row.get("length").unwrap(),
                }
            }
        }
        impl Diff for Blob {
            fn diff(&self, other: &Self) -> Vec<ChangeLog> {
                let mut diff = ::alloc::vec::Vec::new();
                if self.key != other.key {
                    diff.push(ChangeLog {
                        model: "Blob".to_string(),
                        op: "set".to_string(),
                        field: Some("key".to_string()),
                        value: other.key.clone(),
                        ..Default::default()
                    });
                }
                if self.sha256 != other.sha256 {
                    diff.push(ChangeLog {
                        model: "Blob".to_string(),
                        op: "set".to_string(),
                        field: Some("sha256".to_string()),
                        value: Some(other.sha256.clone()),
                        ..Default::default()
                    });
                }
                if self.length != other.length {
                    diff.push(ChangeLog {
                        model: "Blob".to_string(),
                        op: "set".to_string(),
                        field: Some("length".to_string()),
                        value: Some(other.length.to_string()),
                        ..Default::default()
                    });
                }
                diff
            }
            fn apply_diff(&mut self, diff: &[ChangeLog]) {
                for change in diff {
                    if change.op == "set" {
                        if let Some(field) = change.field.clone() {
                            if &field == "key" {
                                self.key = change.value.clone();
                            }
                            if &field == "length" {
                                let src = change.value.clone().unwrap();
                                self.length = u64::from_str_radix(&src, 10).unwrap();
                            }
                            if &field == "sha256" {
                                self.sha256 = change.value.clone().unwrap();
                            }
                        }
                    }
                }
            }
        }
        impl Model for Blob {
            fn table_name() -> String {
                "Blob".to_string()
            }
            fn key(&self) -> Option<String> {
                self.key.clone()
            }
            fn upsert(&self, conn: &rusqlite::Connection) {
                conn.execute(
                        "INSERT OR REPLACE INTO Blob 
            (key, sha256, length) 
            VALUES (?1, ?2, ?3)",
                        (&self.key, &self.sha256, &self.length),
                    )
                    .unwrap();
            }
            fn set_key(&mut self, key: Option<String>) {
                self.key = key.clone();
            }
            fn log_changes() -> bool {
                true
            }
        }
    }
    pub use blob::Blob;
    use crate::library::Library;
    pub trait FromRow {
        fn from_row(row: &Row) -> Self;
    }
    pub trait Diff {
        fn diff(&self, other: &Self) -> Vec<ChangeLog>
        where
            Self: Sized;
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
}
pub mod library {
    use std::{
        sync::{Arc, Mutex, RwLock},
        time::{Duration, Instant},
    };
    use log::info;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use rusqlite::{backup::Backup, Connection, OptionalExtension};
    use symphonia::core::meta::StandardTagKey;
    use ulid::Generator;
    use uuid::Uuid;
    use crate::{
        model::{
            Blob, ChangeLog, FromRow, MediaFile, Model, Playlist, Track, TrackSource,
        },
        scanner::media_file::ScannedFile, sync::Sync,
    };
    pub struct Library {
        _conn: Arc<Mutex<Connection>>,
        database_path: String,
        ulids: Arc<Mutex<Generator>>,
        synchronizers: Arc<RwLock<Vec<Sync>>>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Library {
        #[inline]
        fn clone(&self) -> Library {
            Library {
                _conn: ::core::clone::Clone::clone(&self._conn),
                database_path: ::core::clone::Clone::clone(&self.database_path),
                ulids: ::core::clone::Clone::clone(&self.ulids),
                synchronizers: ::core::clone::Clone::clone(&self.synchronizers),
            }
        }
    }
    /// TODO change notifications
    /// TODO start changing to Release and friends to easier port to GUI
    impl Library {
        /// Open the library located at the specified path. The path is to an
        /// optionally existing Sqlite database. Blobs will be stored in the
        /// same directory as the specified file.
        pub fn open(database_path: &str) -> Self {
            let conn = Connection::open(database_path).unwrap();
            let schema = "PRAGMA journal_mode=WAL;\n\nCREATE TABLE IF NOT EXISTS Metadata (\n    key       TEXT PRIMARY KEY,\n    value     TEXT\n);\n\nCREATE TABLE IF NOT EXISTS Blob (\n    key        TEXT PRIMARY KEY,\n    sha256     TEXT UNIQUE NOT NULL,\n    length     U64 NOT NULL\n);\n\nCREATE TABLE IF NOT EXISTS Artist (\n    key       TEXT PRIMARY KEY,\n    name      TEXT\n);\n\nCREATE TABLE IF NOT EXISTS Track (\n    key       TEXT PRIMARY KEY,\n    artist    TEXT,\n    album     TEXT,\n    title     TEXT,\n    liked     BOOL NOT NULL DEFAULT false\n);\nCREATE INDEX IF NOT EXISTS Track_idx_1 ON Track (artist, album, title);\n\nCREATE TABLE IF NOT EXISTS MediaFile (\n    key       TEXT PRIMARY KEY,\n    file_path TEXT UNIQUE NOT NULL,\n    sha256    TEXT NOT NULL,\n    artist    TEXT,\n    album     TEXT,\n    title     TEXT\n);\n\n-- Note because I keep forgetting it myself: There can be multiple TrackSources\n-- with the same media_file_key for various reasons. For example, a greatest\n-- hits may include the exact recording from the original hit and would thus\n-- reference the same piece of media.\nCREATE TABLE IF NOT EXISTS TrackSource (\n    key            TEXT PRIMARY KEY,\n    track_key      TEXT NOT NULL,\n    blob_key       TEXT\n    -- TODO blobs, urls, etc.\n    -- TODO probably unique across that plus track_key\n    -- FOREIGN KEY (track_key) REFERENCES Track(key) -- TODO breaks a test cause no tracks exist\n);\nCREATE INDEX IF NOT EXISTS TrackSource_idx_1 ON TrackSource (blob_key);\n\nCREATE TABLE IF NOT EXISTS Playlist (\n    key       TEXT PRIMARY KEY,\n    name      TEXT\n);\n\nCREATE TABLE IF NOT EXISTS PlaylistItem (\n    key          TEXT PRIMARY KEY,\n    -- TODO ordinal, probably\n    playlist_key TEXT NOT NULL,\n    track_key    TEXT NOT NULL,\n    FOREIGN KEY (playlist_key) REFERENCES Playlist(key),\n    FOREIGN KEY (track_key) REFERENCES Track(key)\n);\n\nCREATE TABLE IF NOT EXISTS ChangeLog (\n    key       TEXT UNIQUE,\n    actor     TEXT NOT NULL,\n    timestamp TEXT NOT NULL,\n    model     TEXT NOT NULL,\n    model_key TEXT NOT NULL,\n    op        TEXT NOT NULL,\n    field     TEXT,\n    value     TEXT,\n    PRIMARY KEY (actor, timestamp)\n);\nCREATE INDEX IF NOT EXISTS ChangeLog_idx_1 ON ChangeLog (model, model_key, field);\n\n";
            conn.execute_batch(schema).unwrap();
            conn.execute(
                    "
            INSERT INTO Metadata (key, value) VALUES ('library.uuid', ?1)
            ON CONFLICT DO NOTHING
            ",
                    (Uuid::new_v4().to_string(),),
                )
                .unwrap();
            let library = Library {
                _conn: Arc::new(Mutex::new(conn)),
                database_path: database_path.to_string(),
                ulids: Arc::new(Mutex::new(Generator::new())),
                synchronizers: Arc::new(RwLock::new(::alloc::vec::Vec::new())),
            };
            library
        }
        pub fn conn(&self) -> Connection {
            Connection::open(self.database_path.clone()).unwrap()
        }
        /// Returns the unique, permanent ID of this Library. This is created when
        /// the Library is created and doesn't change.
        pub fn id(&self) -> String {
            self.conn()
                .query_row(
                    "SELECT value FROM Metadata WHERE key = 'library.uuid'",
                    (),
                    |row| {
                        let s: String = row.get(0).unwrap();
                        Ok(s)
                    },
                )
                .unwrap()
        }
        /// Backup this library to the specified path.
        pub fn backup(&self, output_path: &str) {
            let mut dst = Connection::open(output_path).unwrap();
            let src = self.conn();
            let backup = Backup::new(&src, &mut dst).unwrap();
            backup.run_to_completion(250, Duration::from_millis(10), None).unwrap();
        }
        /// Import MediaFiles into the Library, creating or updating Tracks,
        /// TrackSources, Blobs, etc.
        /// TODO okay this is slow cause we are scanning all the files first no
        /// matter what, reading all their tags and images and shit, and we might
        /// just ignore that file based on it's sha, so fix that.
        pub fn import(&self, input: &[crate::scanner::media_file::ScannedFile]) {
            let library = self.clone();
            input
                .par_iter()
                .for_each(|input| {
                    library.import_internal(input);
                });
        }
        fn import_internal(&self, input: &ScannedFile) {
            let file_path = std::fs::canonicalize(&input.path).unwrap();
            let file_path = file_path.to_str().unwrap();
            let blob = Blob::read(file_path);
            let blob = self
                .find_blob_by_sha256(&blob.sha256)
                .or_else(|| Some(self.save(&blob)))
                .unwrap();
            let media_file = self
                .find_media_file_by_file_path(file_path)
                .or_else(|| Some(
                    self
                        .save(
                            &MediaFile {
                                file_path: file_path.to_owned(),
                                sha256: blob.sha256.clone(),
                                artist: input.tag(StandardTagKey::Artist),
                                album: input.tag(StandardTagKey::Album),
                                title: input.tag(StandardTagKey::TrackTitle),
                                ..Default::default()
                            },
                        ),
                ))
                .unwrap();
            if self.track_sources_by_blob(&blob).is_empty() {
                let track = self
                    .find_track_for_media_file(&media_file)
                    .or_else(|| Some(
                        self
                            .save(
                                &Track {
                                    artist: media_file.artist,
                                    album: media_file.album,
                                    title: media_file.title,
                                    ..Default::default()
                                },
                            ),
                    ))
                    .unwrap();
                let _source = self
                    .save(
                        &TrackSource {
                            track_key: track.key.unwrap(),
                            blob_key: blob.key.clone(),
                            ..Default::default()
                        },
                    );
            }
        }
        pub fn add_sync(&self, sync: Sync) {
            self.synchronizers.write().unwrap().push(sync);
        }
        pub fn sync(&self) {
            if let Ok(syncs) = self.synchronizers.read() {
                for sync in syncs.iter() {
                    sync.sync(self);
                }
            }
        }
        /// Generates a ulid that is guaranteed to be monotonic.
        pub fn ulid(&self) -> String {
            self.ulids.lock().unwrap().generate().unwrap().to_string()
        }
        pub fn save<T: Model>(&self, obj: &T) -> T {
            let old: T = obj
                .key()
                .as_ref()
                .and_then(|key| self.get(&key))
                .or_else(|| Some(T::default()))
                .unwrap();
            let key = obj.key().or_else(|| Some(Uuid::new_v4().to_string()));
            let mut obj = obj.clone();
            obj.set_key(key.clone());
            obj.upsert(&self.conn());
            let new: T = self.get(&key.unwrap()).unwrap();
            if T::log_changes() {
                let diff = old.diff(&new);
                for mut change in diff {
                    change.timestamp = self.ulid();
                    change.actor = self.id();
                    change.model_key = new.key().clone().unwrap();
                    self.save(&change);
                }
            }
            new
        }
        /// TODO I think drop Model and use a trait that excludes Diff and such
        /// to make this more clear. And then I think I can drop Model.log_changes
        pub fn save_unlogged<T: Model>(&self, obj: &T) -> T {
            let key = obj.key().or_else(|| Some(Uuid::new_v4().to_string()));
            let mut obj = obj.clone();
            obj.set_key(key.clone());
            obj.upsert(&self.conn());
            let new: T = self.get(&key.unwrap()).unwrap();
            new
        }
        pub fn get<T: Model>(&self, key: &str) -> Option<T> {
            let sql = {
                let res = ::alloc::fmt::format(
                    format_args!("SELECT * FROM {0} WHERE key = ?1", T::table_name()),
                );
                res
            };
            self.conn()
                .query_row(&sql, (key,), |row| Ok(T::from_row(row)))
                .optional()
                .unwrap()
        }
        pub fn list<T: Model>(&self) -> Vec<T> {
            let sql = {
                let res = ::alloc::fmt::format(
                    format_args!("SELECT * FROM {0}", T::table_name()),
                );
                res
            };
            self.conn()
                .prepare(&sql)
                .unwrap()
                .query_map((), |row| Ok(T::from_row(row)))
                .unwrap()
                .map(|m| m.unwrap())
                .collect()
        }
        pub fn query<T: Model>(&self, sql: &str) -> Vec<T> {
            let conn = self.conn();
            let result = conn
                .prepare(&sql)
                .unwrap()
                .query_map((), |row| Ok(T::from_row(row)))
                .unwrap()
                .map(|m| m.unwrap())
                .collect();
            result
        }
        pub fn changelogs(&self) -> Vec<ChangeLog> {
            self.query("SELECT * FROM ChangeLog ORDER BY timestamp ASC")
        }
        pub fn tracks(&self) -> Vec<Track> {
            self.query("SELECT * FROM Track ORDER BY artist, album, title")
        }
        pub fn playlist_add(&self, playlist: &Playlist, track_key: &str) {
            self.conn()
                .execute(
                    "INSERT INTO PlaylistItem 
            (key, playlist_key, track_key) 
            VALUES (?1, ?2, ?3)",
                    (
                        &Uuid::new_v4().to_string(),
                        playlist.key.clone().unwrap(),
                        track_key,
                    ),
                )
                .unwrap();
        }
        pub fn playlist_clear(&self, playlist: &Playlist) {
            self.conn()
                .execute(
                    "DELETE FROM PlaylistItem
            WHERE playlist_key = ?1",
                    (playlist.key.clone().unwrap(),),
                )
                .unwrap();
        }
        pub fn find_newest_changelog_by_field(
            &self,
            model: &str,
            model_key: &str,
            field: &str,
        ) -> Option<ChangeLog> {
            self.conn()
                .query_row_and_then(
                    "SELECT * FROM ChangeLog 
            WHERE model = ?1 AND model_key = ?2 AND field = ?3
            ORDER BY timestamp DESC",
                    (model, model_key, field),
                    |row| Ok(ChangeLog::from_row(row)),
                )
                .optional()
                .unwrap()
        }
        pub fn find_media_file_by_file_path(
            &self,
            file_path: &str,
        ) -> Option<MediaFile> {
            self.conn()
                .query_row_and_then(
                    "SELECT * FROM MediaFile
            WHERE file_path = ?1",
                    (file_path,),
                    |row| Ok(MediaFile::from_row(row)),
                )
                .optional()
                .unwrap()
        }
        pub fn find_blob_by_sha256(&self, sha256: &str) -> Option<Blob> {
            self.conn()
                .query_row_and_then(
                    "SELECT * FROM Blob
            WHERE sha256 = ?1",
                    (sha256,),
                    |row| Ok(Blob::from_row(row)),
                )
                .optional()
                .unwrap()
        }
        pub fn find_track_for_media_file(
            &self,
            media_file: &MediaFile,
        ) -> Option<Track> {
            self.conn()
                .query_row_and_then(
                    "SELECT * FROM Track
            WHERE artist = ?1 AND album = ?2 AND title = ?3",
                    (
                        media_file.artist.clone(),
                        media_file.album.clone(),
                        media_file.title.clone(),
                    ),
                    |row| Ok(Track::from_row(row)),
                )
                .optional()
                .unwrap()
        }
        pub fn track_sources_for_track(&self, track: &Track) -> Vec<TrackSource> {
            let conn = self.conn();
            let mut stmt = conn
                .prepare("SELECT * FROM TrackSource
            WHERE track_key = ?1")
                .unwrap();
            stmt.query_map([track.key.clone()], |row| Ok(TrackSource::from_row(row)))
                .unwrap()
                .map(|result| result.unwrap())
                .collect()
        }
        pub fn track_sources_by_blob(&self, blob: &Blob) -> Vec<TrackSource> {
            let conn = self.conn();
            let mut stmt = conn
                .prepare("SELECT * FROM TrackSource
            WHERE blob_key = ?1")
                .unwrap();
            stmt.query_map([blob.key.clone()], |row| Ok(TrackSource::from_row(row)))
                .unwrap()
                .map(|result| result.unwrap())
                .collect()
        }
        pub fn media_files_by_sha256(&self, sha256: &str) -> Vec<MediaFile> {
            let conn = self.conn();
            let mut stmt = conn
                .prepare("SELECT * FROM MediaFile
            WHERE sha256 = ?1")
                .unwrap();
            stmt.query_map([sha256], |row| Ok(MediaFile::from_row(row)))
                .unwrap()
                .map(|result| result.unwrap())
                .collect()
        }
        pub fn load_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
            for media_file in self.media_files_by_sha256(&blob.sha256) {
                if let Ok(content) = std::fs::read(&media_file.file_path) {
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "Found blob sha256 {0} at {1}",
                                    blob.sha256,
                                    &media_file.file_path,
                                ),
                                lvl,
                                &(
                                    "dimple_core_nt::library",
                                    "dimple_core_nt::library",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    };
                    return Some(content);
                }
            }
            for sync in self.synchronizers.read().unwrap().iter() {
                if let Some(content) = sync.load_blob_content(blob) {
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!("Found blob sha256 {0} in sync", blob.sha256),
                                lvl,
                                &(
                                    "dimple_core_nt::library",
                                    "dimple_core_nt::library",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    };
                    return Some(content);
                }
            }
            None
        }
        pub fn load_local_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
            for media_file in self.media_files_by_sha256(&blob.sha256) {
                if let Ok(content) = std::fs::read(media_file.file_path) {
                    return Some(content);
                }
            }
            None
        }
        pub fn load_track_content(&self, track: &Track) -> Option<Vec<u8>> {
            for source in self.track_sources_for_track(track) {
                if let Some(blob_key) = source.blob_key {
                    if let Some(blob) = self.get::<Blob>(&blob_key) {
                        if let Some(content) = self.load_blob_content(&blob) {
                            return Some(content);
                        }
                    }
                }
            }
            None
        }
        /// Test that the database matches the combined state of the changelog.
        pub fn verify() {
            ::core::panicking::panic("not yet implemented")
        }
    }
}
pub mod scanner {
    use media_file::ScannedFile;
    use walkdir::WalkDir;
    pub mod media_file {
        use std::fs::File;
        use symphonia::core::{
            formats::FormatOptions, io::MediaSourceStream,
            meta::{MetadataOptions, StandardTagKey, Tag, Visual},
            probe::Hint,
        };
        pub struct ScannedFile {
            pub path: String,
            pub tags: Vec<Tag>,
            pub visuals: Vec<Visual>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ScannedFile {
            #[inline]
            fn clone(&self) -> ScannedFile {
                ScannedFile {
                    path: ::core::clone::Clone::clone(&self.path),
                    tags: ::core::clone::Clone::clone(&self.tags),
                    visuals: ::core::clone::Clone::clone(&self.visuals),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ScannedFile {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "ScannedFile",
                    "path",
                    &self.path,
                    "tags",
                    &self.tags,
                    "visuals",
                    &&self.visuals,
                )
            }
        }
        impl ScannedFile {
            pub fn new(path: &str) -> Result<ScannedFile, String> {
                let path = std::fs::canonicalize(path).unwrap();
                let media_source = File::open(&path).unwrap();
                let media_source_stream = MediaSourceStream::new(
                    Box::new(media_source),
                    Default::default(),
                );
                let meta_opts: MetadataOptions = Default::default();
                let fmt_opts: FormatOptions = Default::default();
                let mut hint = Hint::new();
                if let Some(extension) = path.extension() {
                    hint.with_extension(extension.to_str().unwrap());
                }
                let probed = symphonia::default::get_probe()
                    .format(&hint, media_source_stream, &fmt_opts, &meta_opts);
                if probed.is_err() {
                    return Err("No media found in probe.".to_string());
                }
                let mut probed = probed.unwrap();
                let mut format = probed.format;
                let mut tags: Vec<Tag> = ::alloc::vec::Vec::new();
                let mut visuals: Vec<Visual> = ::alloc::vec::Vec::new();
                if let Some(metadata) = probed.metadata.get() {
                    if let Some(metadata) = metadata.current() {
                        tags.extend(metadata.tags().to_owned());
                        visuals.extend(metadata.visuals().to_owned());
                    }
                }
                let metadata = format.metadata();
                if let Some(metadata) = metadata.current() {
                    tags.extend(metadata.tags().to_owned());
                    visuals.extend(metadata.visuals().to_owned());
                }
                let media_file = ScannedFile {
                    path: path.to_str().unwrap().to_string(),
                    tags,
                    visuals,
                };
                Ok(media_file)
            }
            pub fn tag(&self, key: StandardTagKey) -> Option<String> {
                self.tags
                    .iter()
                    .find_map(|t| {
                        if let Some(std_key) = t.std_key {
                            if std_key == key {
                                return Some(t.value.to_string());
                            }
                        }
                        None
                    })
            }
        }
    }
    pub struct Scanner {}
    impl Scanner {
        pub fn scan_directory(directory_path: &str) -> Vec<media_file::ScannedFile> {
            WalkDir::new(directory_path)
                .into_iter()
                .filter(|dir_entry| dir_entry.is_ok())
                .map(|dir_entry| dir_entry.unwrap())
                .filter(|dir_entry| dir_entry.file_type().is_file())
                .map(|file_entry| ScannedFile::new(file_entry.path().to_str().unwrap()))
                .filter_map(|mf| mf.ok())
                .collect()
        }
    }
}
pub mod play_queue {}
pub mod player {
    use std::{io::Cursor, sync::Arc};
    use playback_rs::{Hint, Song};
    use crate::{library::Library, model::{Model, Playlist}};
    pub struct Player {
        library: Arc<Library>,
    }
    impl Player {
        pub fn new(library: Arc<Library>) -> Player {
            Player { library }
        }
        pub fn play_queue(&self) -> Playlist {
            let key = {
                let res = ::alloc::fmt::format(
                    format_args!("__dimple_system_play_queue_{0}", self.library.id()),
                );
                res
            };
            let mut playlist = match self.library.get::<Playlist>(&key) {
                Some(play_queue) => play_queue,
                None => {
                    self.library
                        .save(
                            &Playlist {
                                key: Some(key.to_string()),
                                ..Default::default()
                            },
                        )
                }
            };
            playlist.hydrate(&self.library);
            playlist
        }
        pub fn play_queue_add(&self, track_key: &str) {
            let playlist = self.play_queue();
            self.library.playlist_add(&playlist, track_key);
        }
        pub fn play_queue_clear(&self) {
            let playlist = self.play_queue();
            self.library.playlist_clear(&playlist);
        }
        pub fn play(&self) {
            let player = playback_rs::Player::new(None).unwrap();
            let play_queue = self.play_queue();
            for track in play_queue.tracks {
                while player.has_next_song() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                let content = self
                    .library
                    .load_track_content(&track)
                    .expect("No valid sources found.");
                let song = Song::new(Box::new(Cursor::new(content)), &Hint::new(), None)
                    .unwrap();
                player.play_song_next(&song, None).unwrap();
            }
            while player.has_current_song() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
pub mod sync {
    /// Sync a Library with a compatible storage target. Allows multiple
    /// devices to share the same library. Designed for S3 but adaptable with
    /// the Storage trait.
    ///
    /// The sync protocol is designed as a multi-writer distributed operation log.
    /// Each change to the database is logged in the ChangeLog table as one or more
    /// operations, along with a ulid logical timestamp and an actor id based on
    /// the library uuid.
    ///
    /// Each actor / client maintains their own copy of the database, including
    /// the ChangeLog. When a library we merge any remote ChangeLogs that are found,
    /// apply any that are newer than those previously observed, and then push the
    /// combined ChangeLog up to the remote. As each actor performs these actions
    /// the individual databases converge to the same values.
    ///
    /// When creating an Sync instance you specify a path on the Storage to use
    /// as a base. This is prepended along with a / for all storage operations.
    ///
    /// By using a guaranteed unique Sync path like a UUID we can store multiple
    /// libraries on the same storage, or we can store shares.
    pub mod storage {
        pub trait Storage: Send + Sync {
            fn put_object(&self, path: &str, contents: &[u8]);
            fn get_object(&self, path: &str) -> Option<Vec<u8>>;
            fn list_objects(&self, path: &str) -> Vec<String>;
        }
    }
    pub mod s3_storage {
        use std::env;
        use s3::{creds::Credentials, Bucket, Region};
        use super::storage::Storage;
        pub struct S3Storage {
            pub access_key: String,
            pub secret_key: String,
            pub region: String,
            pub endpoint: String,
            pub bucket: String,
            pub prefix: String,
        }
        impl S3Storage {
            pub fn new(
                access_key: &str,
                secret_key: &str,
                region: &str,
                endpoint: &str,
                bucket: &str,
                prefix: &str,
            ) -> Self {
                S3Storage {
                    access_key: access_key.to_owned(),
                    secret_key: secret_key.to_owned(),
                    region: region.to_owned(),
                    endpoint: endpoint.to_owned(),
                    bucket: bucket.to_owned(),
                    prefix: prefix.to_owned(),
                }
            }
            fn open_bucket(&self) -> Bucket {
                let credentials = Credentials::new(
                        Some(&self.access_key),
                        Some(&self.secret_key),
                        None,
                        None,
                        None,
                    )
                    .unwrap();
                let region = Region::Custom {
                    region: self.region.to_owned(),
                    endpoint: self.endpoint.to_owned(),
                };
                Bucket::new(&self.bucket, region, credentials).unwrap()
            }
            fn strip_prefix(s: &str, prefix: &str) -> String {
                if s.starts_with(prefix) {
                    return s[prefix.len()..].to_string();
                }
                s.to_string()
            }
        }
        impl Default for S3Storage {
            fn default() -> Self {
                let access_key = env::var("DIMPLE_TEST_S3_ACCESS_KEY")
                    .expect("Missing DIMPLE_TEST_S3_ACCESS_KEY environment variable.");
                let secret_key = env::var("DIMPLE_TEST_S3_SECRET_KEY")
                    .expect("Missing DIMPLE_TEST_S3_SECRET_KEY environment variable.");
                let region = env::var("DIMPLE_TEST_S3_REGION")
                    .expect("Missing DIMPLE_TEST_S3_REGION environment variable.");
                let endpoint = env::var("DIMPLE_TEST_S3_ENDPOINT")
                    .expect("Missing DIMPLE_TEST_S3_ENDPOINT environment variable.");
                let bucket = env::var("DIMPLE_TEST_S3_BUCKET")
                    .expect("Missing DIMPLE_TEST_S3_BUCKET environment variable.");
                let prefix = env::var("DIMPLE_TEST_S3_PREFIX")
                    .expect("Missing DIMPLE_TEST_S3_PREFIX environment variable.");
                Self::new(&access_key, &secret_key, &region, &endpoint, &bucket, &prefix)
            }
        }
        impl Storage for S3Storage {
            fn put_object(&self, path: &str, contents: &[u8]) {
                let bucket = self.open_bucket();
                bucket.put_object(path, contents).unwrap();
            }
            fn get_object(&self, path: &str) -> Option<Vec<u8>> {
                let bucket = self.open_bucket();
                let obj = bucket.get_object(&path).ok().map(|r| r.to_vec());
                obj
            }
            fn list_objects(&self, path: &str) -> Vec<String> {
                let bucket = self.open_bucket();
                let results = bucket
                    .list(path.to_string(), None)
                    .unwrap()
                    .iter()
                    .flat_map(|r| r.contents.iter())
                    .map(|r| r.key.to_owned())
                    .collect();
                results
            }
        }
    }
    pub mod memory_storage {
        use std::{collections::HashMap, sync::{Arc, RwLock}};
        use super::storage::Storage;
        pub struct MemoryStorage {
            map: Arc<RwLock<HashMap<String, Vec<u8>>>>,
        }
        #[automatically_derived]
        impl ::core::default::Default for MemoryStorage {
            #[inline]
            fn default() -> MemoryStorage {
                MemoryStorage {
                    map: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for MemoryStorage {
            #[inline]
            fn clone(&self) -> MemoryStorage {
                MemoryStorage {
                    map: ::core::clone::Clone::clone(&self.map),
                }
            }
        }
        impl Storage for MemoryStorage {
            fn put_object(&self, path: &str, contents: &[u8]) {
                self.map.write().unwrap().insert(path.to_owned(), contents.to_vec());
            }
            fn get_object(&self, path: &str) -> Option<Vec<u8>> {
                let obj = self.map.read().unwrap().get(path).cloned();
                obj
            }
            fn list_objects(&self, storage_prefix: &str) -> Vec<String> {
                self.map
                    .read()
                    .unwrap()
                    .keys()
                    .filter(|key| key.starts_with(storage_prefix))
                    .cloned()
                    .collect()
            }
        }
    }
    use std::collections::HashSet;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use log::{info, warn};
    use storage::Storage;
    use tempfile::tempdir;
    use uuid::Uuid;
    use crate::{
        library::Library, model::{Blob, ChangeLog, Diff, Model, Track, TrackSource},
    };
    pub struct Sync {
        storage: Box<dyn Storage>,
        path: String,
    }
    impl Sync {
        pub fn new(storage: Box<dyn Storage>, path: &str) -> Self {
            Sync {
                storage,
                path: path.to_string(),
            }
        }
        /// TODO library will need to maintain a reference to sync for looking up
        ///      blobs when it's online.
        ///
        /// # Goals
        ///
        /// - Sync multiple devices via one S3 prefix.
        /// - Create share URLs that allow anyone with the URL to add / listen in
        ///   Dimple. This involves creating pre-signed URLs for partial databases
        ///   and the associated blobs.
        ///
        ///
        /// # File Layout
        ///
        /// - {path}/db/{library.id()}.db
        ///   Databases of devices participating in the sync.
        ///
        /// - {path}/blobs/{blob.sha256}.blob
        ///   Blobs stored under their SHA256 for de-dupe. This includes media,
        ///   images, cover art, etc.
        ///
        /// - {path}/shares/{share.id()}.db
        ///   Database of shared info for a specific share. Shared via pre-signed URL
        ///   and includes pre-signed URLs to reference the blobs.
        ///
        /// I think this is actually going to reflect the layout on local disk too.
        ///
        pub fn sync(&self, library: &Library) {
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("Synchronizing {0}.", library.id()),
                        lvl,
                        &(
                            "dimple_core_nt::sync",
                            "dimple_core_nt::sync",
                            ::log::__private_api::loc(),
                        ),
                        (),
                    );
                }
            };
            let temp_dir = tempdir().unwrap();
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Pulling remote changes."),
                            lvl,
                            &(
                                "dimple_core_nt::sync",
                                "dimple_core_nt::sync",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                let remote_library_paths = self
                    .storage
                    .list_objects(
                        &{
                            let res = ::alloc::fmt::format(
                                format_args!("{0}/db/", self.path),
                            );
                            res
                        },
                    );
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Remote libraries {0:?}", remote_library_paths),
                            lvl,
                            &(
                                "dimple_core_nt::sync",
                                "dimple_core_nt::sync",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                remote_library_paths
                    .iter()
                    .for_each(|remote_library_path| {
                        let contents = self
                            .storage
                            .get_object(remote_library_path)
                            .unwrap();
                        let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!(
                                        "Downloading {0} to {1}.",
                                        remote_library_path,
                                        temp_file.to_str().unwrap(),
                                    ),
                                    lvl,
                                    &(
                                        "dimple_core_nt::sync",
                                        "dimple_core_nt::sync",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        };
                        std::fs::write(&temp_file, &contents).unwrap();
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!(
                                        "Opening library {0}.",
                                        temp_file.to_str().unwrap(),
                                    ),
                                    lvl,
                                    &(
                                        "dimple_core_nt::sync",
                                        "dimple_core_nt::sync",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        };
                        let remote_library = Library::open(temp_file.to_str().unwrap());
                        if remote_library.id() == library.id() {
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("Skipping own library with same id."),
                                        lvl,
                                        &(
                                            "dimple_core_nt::sync",
                                            "dimple_core_nt::sync",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            };
                            return;
                        }
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!(
                                        "Library contains {0} tracks and {1} changelogs.",
                                        remote_library.tracks().len(),
                                        remote_library.changelogs().len(),
                                    ),
                                    lvl,
                                    &(
                                        "dimple_core_nt::sync",
                                        "dimple_core_nt::sync",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        };
                        let changelogs = remote_library.changelogs();
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!("Applying {0} changelogs", changelogs.len()),
                                    lvl,
                                    &(
                                        "dimple_core_nt::sync",
                                        "dimple_core_nt::sync",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        };
                        for changelog in changelogs {
                            Self::apply_changelog(library, &changelog);
                            library.save(&changelog);
                        }
                    });
            }
            {
                let temp_file = temp_dir.path().join(Uuid::new_v4().to_string());
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Gathering local changes."),
                            lvl,
                            &(
                                "dimple_core_nt::sync",
                                "dimple_core_nt::sync",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                library.backup(temp_file.to_str().unwrap());
                let path = {
                    let res = ::alloc::fmt::format(
                        format_args!("{0}/db/{1}.db", self.path, library.id()),
                    );
                    res
                };
                let contents = std::fs::read(temp_file).unwrap();
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Pushing local changes."),
                            lvl,
                            &(
                                "dimple_core_nt::sync",
                                "dimple_core_nt::sync",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                self.storage.put_object(&path, &contents);
            }
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Syncing blobs"),
                            lvl,
                            &(
                                "dimple_core_nt::sync",
                                "dimple_core_nt::sync",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                let local_blobs: Vec<Blob> = library.list::<Blob>();
                let remote_blob_names: HashSet<String> = self
                    .storage
                    .list_objects(
                        &{
                            let res = ::alloc::fmt::format(
                                format_args!("{0}/blobs/", self.path),
                            );
                            res
                        },
                    )
                    .iter()
                    .map(|n| n.rsplit_once("/").unwrap().1.to_string())
                    .collect();
                let to_store: Vec<Blob> = local_blobs
                    .into_iter()
                    .filter(|b| {
                        !remote_blob_names
                            .contains(
                                &{
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}.blob", b.sha256),
                                    );
                                    res
                                },
                            )
                    })
                    .collect();
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Pushing {0} new blobs.", to_store.len()),
                            lvl,
                            &(
                                "dimple_core_nt::sync",
                                "dimple_core_nt::sync",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                to_store
                    .par_iter()
                    .for_each(|blob| {
                        if let Some(content) = library.load_local_blob_content(&blob) {
                            let path = {
                                let res = ::alloc::fmt::format(
                                    format_args!("{0}/blobs/{1}.blob", self.path, blob.sha256),
                                );
                                res
                            };
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("Pushing blob {0}.", path),
                                        lvl,
                                        &(
                                            "dimple_core_nt::sync",
                                            "dimple_core_nt::sync",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            };
                            self.storage.put_object(&path, &content);
                        } else {
                            {
                                let lvl = ::log::Level::Warn;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!(
                                            "No content found to sync for sha256 {0}",
                                            blob.sha256,
                                        ),
                                        lvl,
                                        &(
                                            "dimple_core_nt::sync",
                                            "dimple_core_nt::sync",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            };
                        }
                    });
            }
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("Sync complete."),
                        lvl,
                        &(
                            "dimple_core_nt::sync",
                            "dimple_core_nt::sync",
                            ::log::__private_api::loc(),
                        ),
                        (),
                    );
                }
            };
        }
        pub fn load_blob_content(&self, blob: &Blob) -> Option<Vec<u8>> {
            let path = {
                let res = ::alloc::fmt::format(
                    format_args!("{0}/blobs/{1}.blob", self.path, blob.sha256),
                );
                res
            };
            self.storage.get_object(&path)
        }
        fn apply_changelog(library: &Library, changelog: &ChangeLog) {
            let actor = changelog.actor.clone();
            let timestamp = changelog.timestamp.clone();
            let model = changelog.model.clone();
            let model_key = changelog.model_key.clone();
            let op = changelog.op.clone();
            if actor == library.id() {
                return;
            }
            if model == "Track" {
                if op == "set" {
                    let field = changelog.field.clone().unwrap();
                    if let Some(newest_changelog) = library
                        .find_newest_changelog_by_field(&model, &model_key, &field)
                    {
                        if newest_changelog.timestamp >= timestamp {
                            return;
                        }
                    }
                    let mut obj = library
                        .get(&model_key)
                        .or_else(|| Some(Track {
                            key: Some(model_key.clone()),
                            ..Default::default()
                        }))
                        .unwrap();
                    obj.apply_diff(&[changelog.clone()]);
                    library.save_unlogged(&obj);
                }
            }
            if model == "TrackSource" {
                if op == "set" {
                    let field = changelog.field.clone().unwrap();
                    if let Some(newest_changelog) = library
                        .find_newest_changelog_by_field(&model, &model_key, &field)
                    {
                        if newest_changelog.timestamp >= timestamp {
                            return;
                        }
                    }
                    let mut obj = library
                        .get(&model_key)
                        .or_else(|| Some(TrackSource {
                            key: Some(model_key.clone()),
                            ..Default::default()
                        }))
                        .unwrap();
                    obj.apply_diff(&[changelog.clone()]);
                    library.save_unlogged(&obj);
                }
            }
            if model == "Blob" {
                if op == "set" {
                    let field = changelog.field.clone().unwrap();
                    if let Some(newest_changelog) = library
                        .find_newest_changelog_by_field(&model, &model_key, &field)
                    {
                        if newest_changelog.timestamp >= timestamp {
                            return;
                        }
                    }
                    let mut obj = library
                        .get(&model_key)
                        .or_else(|| Some(Blob {
                            key: Some(model_key.clone()),
                            ..Default::default()
                        }))
                        .unwrap();
                    obj.apply_diff(&[changelog.clone()]);
                    library.save_unlogged(&obj);
                }
            }
        }
    }
}
