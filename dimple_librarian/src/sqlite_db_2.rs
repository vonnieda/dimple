use anyhow::Error;
use dimple_core::model::Entity;
use rusqlite::{Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

pub struct SqliteDb2 {
    path: String,
    connection: Connection,
}

impl SqliteDb2 {
    pub fn new(path: &str) -> Result<Self, Error> {
        let conn =  Connection::open(path)?;

        conn.execute(
                "CREATE TABLE IF NOT EXISTS Artist (
                key UUID PRIMARY KEY NOT NULL,
                doc JSON NOT NULL)",
                (),
            )
            .unwrap();
        
        Ok(Self {
            path: path.to_string(),
            connection: conn
        })
    }

    pub fn set<T: Default + Serialize + Entity + Clone>(&self, model: &T) -> Result<T, Error> {
        let mut model = model.clone();
        if model.key().is_none() {
            model.set_key(Some(Uuid::new_v4().to_string()));
        }
        let doc = serde_json::to_string_pretty(&model)?;
        self.connection.execute(
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
        let mut stmt = self.connection.prepare(query)?;

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
        let _ = self.connection.execute("BEGIN", [])?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), Error> {
        let _ = self.connection.execute("COMMIT", [])?;
        Ok(())
    }

    pub fn rollback(&self) -> Result<(), Error> {
        let _ = self.connection.execute("ROLLBACK", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dimple_core::model::Artist;
    use rusqlite::Connection;

    use crate::sqlite_db_2::SqliteDb2;

    #[test]
    fn basics() {
        let db = SqliteDb2::new(":memory:").unwrap();
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
        let db = SqliteDb2::new(":memory:").unwrap();
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
        let db = SqliteDb2::new(":memory:").unwrap();

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

        let artists: Vec<Artist> = db.query("SELECT doc FROM Artist")
            .unwrap().collect();
        assert!(artists.len() == 4);
        let artists: Vec<Artist> = db.query("SELECT doc FROM Artist WHERE doc->>'name' LIKE 'say h%'")
            .unwrap().collect();
        assert!(artists.len() == 3);
    }

    #[test]
    fn transactions() {
        let db = SqliteDb2::new(":memory:").unwrap();

        db.begin().unwrap();
        db.set(&Artist {
            key: Some("3cbd37cd-e019-430e-90eb-6ef35a4e1b70".to_string()),
            name: Some("say hi".to_string()),
            ..Default::default()
        }).unwrap();
        db.rollback().unwrap();
        let artist: Option<Artist> = db.get("3cbd37cd-e019-430e-90eb-6ef35a4e1b70").unwrap();
        assert!(artist.is_none());

        db.begin().unwrap();
        db.set(&Artist {
            key: Some("3cbd37cd-e019-430e-90eb-6ef35a4e1b70".to_string()),
            name: Some("say howdy".to_string()),
            ..Default::default()
        }).unwrap();
        db.commit().unwrap();
        let artist: Artist = db.get("3cbd37cd-e019-430e-90eb-6ef35a4e1b70").unwrap().unwrap();
        assert!(artist.name == Some("say howdy".to_string()));
    }

    #[test]
    fn multiple_connections() {
        let path = ".multiple_connections_test.db";
        let _ = std::fs::remove_file(path);
        let conn_1 = Connection::open(path).unwrap();
        let conn_2 = Connection::open(path).unwrap();
        conn_1
            .execute(
                "CREATE TABLE IF NOT EXISTS Artist (
                key UUID PRIMARY KEY,
                doc JSON NOT NULL)",
                (),
            )
            .unwrap();
        conn_2
            .execute(
                "REPLACE INTO Artist VALUES ('d5c8485a-af93-4ae0-8868-b925a7db486b', '{}')",
                (),
            )
            .unwrap();
        let mut stmt = conn_1.prepare("SELECT key, doc FROM Artist").unwrap();
        let iter = stmt
            .query_map([], |row| {
                let key: String = row.get(0).unwrap();
                let doc: String = row.get(1).unwrap();
                Ok(key)
            })
            .unwrap();
        assert!(iter.count() == 1);
        drop(stmt);
        conn_1.close().unwrap();
        conn_2.close().unwrap();
        let _ = std::fs::remove_file(path);
    }
}
