use std::sync::{Arc, Mutex, RwLock};

use anyhow::Error;
use dimple_core::{
    db::Db,
    model::{
        Artist, ArtistCredit, Blob, Dimage, Entity, Genre, KnownIds, Lyrics, Medium, Model, Playlist, PlaylistItem, Recording, RecordingSource, Release, ReleaseGroup, Tag, Track
    },
};
use rusqlite::{Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

pub struct SqliteDb {
    connection: Mutex<Connection>,
}

// https://docs.rs/rusqlite/latest/rusqlite/types/index.html
impl SqliteDb {
    pub fn new(path: &str) -> Result<Self, Error> {
        // TODO store the path and have a function that returns a connection,
        // instead of sharing the connection. Leads to connection pool if we
        // need that.
        // Seems more and more like I should have just implemented like ToSql
        // and FromSql for the types, and then just used rusqlite directly in
        // the librarian.
        let conn = Connection::open(path)?;

        Self::create_collection(&conn, Artist::default())?;
        Self::create_collection(&conn, ArtistCredit::default())?;
        Self::create_collection(&conn, Blob::default())?;
        Self::create_collection(&conn, Dimage::default())?;
        Self::create_collection(&conn, Genre::default())?;
        Self::create_collection(&conn, Lyrics::default())?;
        Self::create_collection(&conn, Medium::default())?;
        Self::create_collection(&conn, PlaylistItem::default())?;
        Self::create_collection(&conn, Playlist::default())?;
        Self::create_collection(&conn, RecordingSource::default())?;
        Self::create_collection(&conn, Recording::default())?;
        Self::create_collection(&conn, ReleaseGroup::default())?;
        Self::create_collection(&conn, Release::default())?;
        Self::create_collection(&conn, Tag::default())?;
        Self::create_collection(&conn, Track::default())?;

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS Edges (
            key_from UUID NOT NULL,
            key_to UUID NOT NULL,
            PRIMARY KEY (key_from, key_to)) 
            "
        );
        conn.execute(&sql, ())?;

        Ok(Self {
            connection: Mutex::new(conn),
        })
    }

    fn create_collection<T: Entity>(conn: &Connection, entity: T) -> Result<(), Error> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
            key UUID PRIMARY KEY NOT NULL,
            doc JSON NOT NULL)",
            entity.type_name()
        );
        conn.execute(&sql, ())?;
        Ok(())
    }

    pub fn set<T: Default + Serialize + DeserializeOwned + Entity + Clone>(&self, model: &T) -> Result<T, Error> {
        let mut model = model.clone();
        if model.key().is_none() {
            model.set_key(Some(Uuid::new_v4().to_string()));
        }
        let doc = serde_json::to_string_pretty(&model)?;
        self.connection.lock().unwrap().execute(
            &format!(
                "REPLACE INTO {} (key, doc) VALUES (?, json(?))",
                model.type_name()
            ),
            [model.key(), Some(doc)],
        )?;
        self.get(&model.key().unwrap()).map(|o| o.unwrap())
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: Default + Entity + Clone + DeserializeOwned,
    {
        let type_name = T::default().type_name();

        let doc = self
            .connection
            .lock()
            .unwrap()
            .query_row(
                &format!("SELECT doc FROM {} WHERE key = ?", type_name),
                [key.to_string()],
                |row| {
                    let doc: String = row.get(0).unwrap();
                    Ok(doc)
                },
            )
            .optional()?;

        if let Some(doc) = doc {
            let result: T = serde_json::from_str(&doc)?;
            return Ok(Some(result));
        }

        Ok(None)
    }

    pub fn query<T>(&self, query: &str) -> Result<Box<dyn Iterator<Item = T>>, Error>
    where
        T: Default + Clone + DeserializeOwned + 'static,
    {
        let binding = self.connection.lock().unwrap();
        let mut stmt = binding.prepare(query)?;

        // TODO would love to return the iterator directly, but need to figure
        // out lifetime stuff.
        let results: Vec<T> = stmt
            .query_map([], |row| {
                let doc: String = row.get(0)?;
                // TODO Fix up this error handling.
                let result: T = serde_json::from_str(&doc).unwrap();
                Ok(result)
            })?
            .map(|row| row.unwrap())
            .collect();

        Ok(Box::new(results.into_iter()))
    }

    pub fn begin(&self) -> Result<(), Error> {
        let _ = self.connection.lock().unwrap().execute("BEGIN", ())?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), Error> {
        let _ = self.connection.lock().unwrap().execute("COMMIT", ())?;
        Ok(())
    }

    pub fn rollback(&self) -> Result<(), Error> {
        let _ = self.connection.lock().unwrap().execute("ROLLBACK", ())?;
        Ok(())
    }
}

impl Db for SqliteDb {
    fn insert(&self, model: &Model) -> anyhow::Result<Model> {
        let mut model = model.clone();
        if model.entity().key().is_none() {
            model.set_key(Some(Uuid::new_v4().to_string()));
        }
        let doc = serde_json::to_string_pretty(&model)?;
        self.connection.lock().unwrap().execute(
            &format!(
                "REPLACE INTO {} (key, doc) VALUES (?, json(?))",
                model.entity().type_name()
            ),
            [model.entity().key(), Some(doc)],
        )?;
        Ok(model)
    }

    fn get(&self, model: &Model) -> anyhow::Result<Option<Model>> {
        let query = format!("SELECT doc FROM {} WHERE key = ?", model.entity().type_name());
        let binding = self.connection.lock().unwrap();
        let mut stmt = binding.prepare(&query)?;
        let model = stmt.query_row([model.entity().key().unwrap()], |row| {
                let doc: String = row.get(0)?;
                let result: Model = serde_json::from_str(&doc).unwrap();
                Ok(result)
            })?;
        Ok(Some(model))
    }

    fn link(&self, model: &Model, related_to: &Model) -> anyhow::Result<()> {
        let key_from = model.entity().key().unwrap();
        let key_to = related_to.entity().key().unwrap();
        self.connection.lock().unwrap().execute(
            "REPLACE INTO Edges (key_from, key_to) VALUES (?, ?)",
            [&key_from, &key_to],
        )?;
        self.connection.lock().unwrap().execute(
            "REPLACE INTO Edges (key_from, key_to) VALUES (?, ?)",
            [&key_to, &key_from],
        )?;
        Ok(())
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = Model>>> {
        let binding = self.connection.lock().unwrap();
        let results = match related_to {
            None => {
                let query = format!("SELECT doc FROM {}", list_of.entity().type_name());
                let mut stmt = binding.prepare(&query)?;
                let results: Vec<Model> = stmt
                    .query_map((), |row| {
                        let doc: String = row.get(0)?;
                        let result: Model = serde_json::from_str(&doc).unwrap();
                        Ok(result)
                    })?
                    .map(|row| row.unwrap())
                    .collect();
                results
            },
            Some(related_to) => {
                let query = format!(
                    "SELECT list_of.doc FROM {} AS list_of 
                    JOIN Edges AS related_to 
                    ON list_of.key = related_to.key_to 
                    WHERE related_to.key_from = ?",
                    list_of.entity().type_name()
                );
                let mut stmt = binding.prepare(&query)?;
                let results: Vec<Model> = stmt
                    .query_map((related_to.entity().key().unwrap(), ), |row| {
                        let doc: String = row.get(0)?;
                        let result: Model = serde_json::from_str(&doc).unwrap();
                        Ok(result)
                    })?
                    .map(|row| row.unwrap())
                    .collect();
                results
            }
        };

        Ok(Box::new(results.into_iter()))
    }

    fn query(
        &self,
        query: &str,
    ) -> anyhow::Result<Box<dyn Iterator<Item = Model>>> {
        let binding = self.connection.lock().unwrap();
        let mut stmt = binding.prepare(&query)?;
        let results: Vec<Model> = stmt
            .query_map((), |row| {
                let doc: String = row.get(0)?;
                let result: Model = serde_json::from_str(&doc).unwrap();
                Ok(result)
            })?
            .map(|row| row.unwrap())
            .collect();
        Ok(Box::new(results.into_iter()))
    }


    fn reset(&self) -> anyhow::Result<()> {
        todo!()
    }
}

