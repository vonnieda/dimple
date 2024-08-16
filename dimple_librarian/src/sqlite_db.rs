
use anyhow::Error;
use dimple_core::{
    db::Db,
    model::{
        Artist, ArtistCredit, Blob, Dimage, Entity, Genre, KnownIds, Lyrics, Medium, Model, Playlist, PlaylistItem, Recording, RecordingSource, Release, ReleaseGroup, Tag, Track
    },
};
use rusqlite::Connection;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteDb {
    db_path: String,
}

// https://docs.rs/rusqlite/latest/rusqlite/types/index.html
impl SqliteDb {
    pub fn new(path: &str) -> Result<Self, Error> {
        let db = Self {
            db_path: path.to_string(),
        };

        let conn = db.get_connection()?;

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
        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS Edges (
            key_from UUID NOT NULL,
            key_to UUID NOT NULL,
            PRIMARY KEY (key_from, key_to)) 
            ", 
        ())?;    

        Ok(db)
    }

    pub fn get_connection(&self) -> Result<Connection, Error> {
        Ok(Connection::open(self.db_path.clone())?)
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


    // pub fn set<T: Default + Serialize + DeserializeOwned + Entity + Clone>(&self, model: &T) -> Result<T, Error> {
    //     let mut model = model.clone();
    //     if model.key().is_none() {
    //         model.set_key(Some(Uuid::new_v4().to_string()));
    //     }
    //     let doc = serde_json::to_string_pretty(&model)?;
    //     self.get_connection()?.execute(
    //         &format!(
    //             "REPLACE INTO {} (key, doc) VALUES (?, json(?))",
    //             model.type_name()
    //         ),
    //         [model.key(), Some(doc)],
    //     )?;
    //     self.get(&model.key().unwrap()).map(|o| o.unwrap())
    // }

    // pub fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    // where
    //     T: Default + Entity + Clone + DeserializeOwned,
    // {
    //     let type_name = T::default().type_name();

    //     let doc = self.get_connection()?
    //         .query_row(
    //             &format!("SELECT doc FROM {} WHERE key = ?", type_name),
    //             [key.to_string()],
    //             |row| {
    //                 let doc: String = row.get(0).unwrap();
    //                 Ok(doc)
    //             },
    //         )
    //         .optional()?;

    //     if let Some(doc) = doc {
    //         let result: T = serde_json::from_str(&doc)?;
    //         return Ok(Some(result));
    //     }

    //     Ok(None)
    // }
}

impl Db for SqliteDb {
    fn insert(&self, model: &Model) -> anyhow::Result<Model> {
        let mut model = model.clone();
        if model.entity().key().is_none() {
            model.set_key(Some(Uuid::new_v4().to_string()));
        }
        let doc = serde_json::to_string_pretty(&model)?;
        self.get_connection()?.execute(
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
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
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
        self.get_connection()?.execute(
            "REPLACE INTO Edges (key_from, key_to) VALUES (?, ?)",
            [&key_from, &key_to],
        )?;
        self.get_connection()?.execute(
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
        let conn = self.get_connection()?;
        let results = match related_to {
            None => {
                let query = format!("SELECT doc FROM {}", list_of.entity().type_name());
                let mut stmt = conn.prepare(&query)?;
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
                let mut stmt = conn.prepare(&query)?;
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
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
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
        // TODO
        todo!()
    }
}

