use std::sync::{Arc, Mutex, RwLock};

use anyhow::Error;
use dimple_core::{db::Db, model::{Artist, ArtistCredit, Blob, Dimage, Entity, Genre, KnownIds, Lyrics, Model, Playlist, PlaylistItem, Recording, RecordingSource, Release, ReleaseGroup, Tag}};
use rusqlite::{Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

pub struct SqliteDb {
    connection: Mutex<Connection>,
}

impl SqliteDb {
    pub fn new(path: &str) -> Result<Self, Error> {
        // TODO store the path and have a function that returns a connection,
        // instead of sharing the connection. Leads to connection pool if we
        // need that.
        let conn = Connection::open(path)?;

        Self::create_collection(&conn, Artist::default())?;
        Self::create_collection(&conn, Blob::default())?;
        Self::create_collection(&conn, Dimage::default())?;
        Self::create_collection(&conn, Genre::default())?;
        Self::create_collection(&conn, Lyrics::default())?;
        Self::create_collection(&conn, PlaylistItem::default())?;
        Self::create_collection(&conn, Playlist::default())?;
        Self::create_collection(&conn, RecordingSource::default())?;
        Self::create_collection(&conn, Recording::default())?;
        Self::create_collection(&conn, ReleaseGroup::default())?;
        Self::create_collection(&conn, Release::default())?;
        Self::create_collection(&conn, Tag::default())?;

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

    pub fn set<T: Serialize + Entity + Clone>(&self, model: &T) -> Result<T, Error> {
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
        Ok(model)
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
        todo!()
    }

    fn get(&self, model: &Model) -> anyhow::Result<Option<Model>> {
        todo!()
    }

    fn link(&self, model: &Model, related_to: &Model) -> anyhow::Result<()> {
        todo!()
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = Model>>> {
        todo!()
    }

    fn reset(&self) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use dimple_core::model::{Artist, Genre};

    use crate::sqlite_db::SqliteDb;

    #[test]
    fn basics() {
        let db = SqliteDb::new(":memory:").unwrap();
        let artist = db
            .set(&Artist {
                name: Some("say hi".to_string()),
                ..Default::default()
            })
            .unwrap();
        let artist2: Artist = db.get(&artist.key.clone().unwrap()).unwrap().unwrap();
        assert!(artist == artist2);
    }

    #[test]
    fn get() {
        let db = SqliteDb::new(":memory:").unwrap();
        let artist = db
            .set(&Artist {
                key: Some("b5965896-9124-41c1-adfc-ea924df70d05".to_string()),
                name: Some("say hi".to_string()),
                ..Default::default()
            })
            .unwrap();

        let artist2: Artist = db
            .get("b5965896-9124-41c1-adfc-ea924df70d05")
            .unwrap()
            .unwrap();
        assert!(artist == artist2);

        let artist3: Option<Artist> = db.get("").unwrap();
        assert!(artist3.is_none());

        let artist4: Option<Artist> = db.get("2d6f8f82-f57d-4f83-ab5f-f13c1471bc17").unwrap();
        assert!(artist4.is_none());
    }

    #[test]
    fn query() {
        let db = SqliteDb::new(":memory:").unwrap();

        db.set(&Artist {
            name: Some("say hi".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.set(&Artist {
            name: Some("say hello".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.set(&Artist {
            name: Some("say howdy".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.set(&Artist {
            name: Some("say yo".to_string()),
            ..Default::default()
        })
        .unwrap();

        let artists: Vec<Artist> = db.query("SELECT doc FROM Artist").unwrap().collect();
        assert!(artists.len() == 4);
        let artists: Vec<Artist> = db
            .query("SELECT doc FROM Artist WHERE doc->>'name' LIKE 'say h%'")
            .unwrap()
            .collect();
        assert!(artists.len() == 3);
    }

    #[test]
    fn transactions() {
        let db = SqliteDb::new(":memory:").unwrap();

        db.begin().unwrap();
        db.set(&Artist {
            key: Some("3cbd37cd-e019-430e-90eb-6ef35a4e1b70".to_string()),
            name: Some("say hi".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.rollback().unwrap();
        let artist: Option<Artist> = db.get("3cbd37cd-e019-430e-90eb-6ef35a4e1b70").unwrap();
        assert!(artist.is_none());

        db.begin().unwrap();
        db.set(&Artist {
            key: Some("3cbd37cd-e019-430e-90eb-6ef35a4e1b70".to_string()),
            name: Some("say howdy".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.commit().unwrap();
        let artist: Artist = db
            .get("3cbd37cd-e019-430e-90eb-6ef35a4e1b70")
            .unwrap()
            .unwrap();
        assert!(artist.name == Some("say howdy".to_string()));
    }

    #[test]
    fn relations() {
        let db = SqliteDb::new(":memory:").unwrap();

        db.set(&Genre {
            key: Some("fa8923db-836f-43ce-92f8-1fcd6eca4adc".to_string()),
            name: Some("Genre 1".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.set(&Genre {
            key: Some("f199bb03-0f37-486e-9b6a-74b026cb17ff".to_string()),
            name: Some("Genre 2".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.set(&Genre {
            key: Some("c74eb140-a673-4890-abbb-e20f7cb66d63".to_string()),
            name: Some("Genre 3".to_string()),
            ..Default::default()
        })
        .unwrap();
        db.set(&Genre {
            key: Some("434f7881-8f88-420f-9320-f299f389e6eb".to_string()),
            name: Some("Genre 4".to_string()),
            ..Default::default()
        })
        .unwrap();

        db.set(&Artist {
            key: Some("fae7a3f6-812e-4372-a8a6-6781e12afa66".to_string()),
            name: Some("Artist 1".to_string()),
            genres: vec![
                Genre {
                    key: Some("fa8923db-836f-43ce-92f8-1fcd6eca4adc".to_string()),
                    ..Default::default()
                },
                Genre {
                    key: Some("c74eb140-a673-4890-abbb-e20f7cb66d63".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .unwrap();

        let genres: Vec<Genre> = db
            .query(
            //     "SELECT doc 
            // FROM Genre AS g
            // WHERE g.key IN (
            //     SELECT json_extract(je.value, '$.key')
            //     FROM Artist AS a, json_each(a.doc, '$.genres') AS je 
            //     WHERE a.key = 'fae7a3f6-812e-4372-a8a6-6781e12afa66')",
                "SELECT g.doc
                FROM Genre AS g
                INNER JOIN (
                    SELECT json_extract(je.value, '$.key') AS genre_key
                    FROM Artist AS a, json_each(a.doc, '$.genres') AS je
                    WHERE a.key = 'fae7a3f6-812e-4372-a8a6-6781e12afa66'
                ) AS extracted_genres
                ON g.key = extracted_genres.genre_key;"
            )
            .unwrap()
            .collect();
        assert!(genres.len() == 2);

        let genres: Vec<Genre> = db.query("SELECT doc FROM Genre").unwrap().collect();
        assert!(genres.len() == 4);
    }

    #[test]
    fn multiple_connections() {
        // This syntax creates an in-memory database that multiple connections
        // can attach to, the same as if it was an actual file.
        // https://www.sqlite.org/inmemorydb.html
        let path = "file:memdb1?mode=memory&cache=shared";
        let db_1 = SqliteDb::new(path).unwrap();
        let db_2 = SqliteDb::new(path).unwrap();
        let db_3 = SqliteDb::new(path).unwrap();
        db_1.set(&Artist::default()).unwrap();
        db_2.set(&Artist::default()).unwrap();
        assert!(
            db_3.query::<Artist>("SELECT doc FROM Artist")
                .unwrap()
                .count()
                == 2
        );
    }
}
