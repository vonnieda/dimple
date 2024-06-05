use std::borrow::BorrowMut;

use anyhow::Result;

use uuid::Uuid;

use crate::model::{Artist, Entity, Genre, Model, Release, Track};

use super::Db;

use sqlite::{Connection, ConnectionThreadSafe, State};

pub struct SqliteDb {
    con: ConnectionThreadSafe,
}

impl SqliteDb {
    pub fn new(path: &str) -> Self {
        let con = Connection::open_thread_safe(path).unwrap();
        con.execute("CREATE TABLE IF NOT EXISTS kv (key TEXT NOT NULL PRIMARY KEY, value BLOB NOT NULL)").unwrap();
        Self {
            con,
        }
    }

    fn _get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut statement = self.con.prepare("SELECT value FROM kv WHERE key = :key")?;
        statement.bind((":key", key))?;
        if let Ok(State::Row) = statement.next() {
            let value = statement.read::<Vec<u8>, _>("value")?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    fn _insert(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut statement = self.con.prepare("REPLACE INTO kv (key, value) VALUES (:key, :value)")?;
        statement.bind((":key", key))?;
        statement.bind((":value", value))?;
        statement.next()?;
        Ok(())
    }

    fn _list(&self, prefix: &str) -> Result<Box<dyn Iterator<Item = Vec<u8>>>> {
        let mut statement = self.con.prepare("SELECT value FROM kv WHERE key LIKE :prefix")?;
        statement.bind((":prefix", prefix))?;
        let mut results: Vec<Vec<u8>> = vec![];
        while let Ok(State::Row) = statement.next() {
            let value = statement.read::<Vec<u8>, _>("value")?;
            results.push(value);
        }
        Ok(Box::new(results.into_iter()))
    }

    fn node_key(model: &Model) -> String {
        // type:key
        format!("node:{}:{}", model.entity().type_name(), 
            model.entity().key().unwrap())
    }

    fn node_prefix(model: &Model) -> String {
        // type:
        format!("node:{}:", model.entity().type_name())
    }

    fn edge_key(model: &Model, related_to: &Model) -> String {
        // edge_key(release, artist) -> atype:btype:akey:bkey
        format!(
            "edge:{}:{}:{}:{}",
            related_to.entity().type_name(),
            related_to.entity().key().unwrap(),
            model.entity().type_name(),
            model.entity().key().unwrap(),
        )
    }

    fn edge_prefix(model: &Model, related_to: &Model) -> String {
        // edge_prefix(release, artist) -> atype:btype:akey:
        format!(
            "edge:{}:{}:{}:",
            related_to.entity().type_name(),
            related_to.entity().key().unwrap(),
            model.entity().type_name(),
        )
    }
}

impl Db for SqliteDb {
    fn insert(&self, model: &Model) -> Result<Model> {
        let model = match model.entity().key() {
            Some(_) => model.clone(),
            None => {
                let mut model = model.clone();
                model.set_key(Some(Uuid::new_v4().to_string()));
                model
            }
        };
        let key = Self::node_key(&model);
        let value = bincode::serialize(&model).unwrap();
        self._insert(&key, &value)?;
        Ok(model)
    }

    fn get(&self, model: &Model) -> Result<Option<Model>> {
        if model.entity().key().is_none() {
            return Ok(None)
        }
        let key = Self::node_key(model);
        let value = self._get(&key)?;
        if value.is_none() {
            return Ok(None);
        }
        let model: Model = bincode::deserialize(&value.unwrap()).unwrap();
        Ok(Some(model))
    }

    fn link(&self, model: &Model, related_to: &Model) -> Result<()> {
        // related_to -> model
        let key = Self::edge_key(model, related_to);
        let related_key = Self::node_key(model);
        self._insert(&key, related_key.as_bytes())?;

        // model -> related_to
        let key = Self::edge_key(related_to, model);
        let related_key = Self::node_key(related_to);
        self._insert(&key, related_key.as_bytes())?;

        Ok(())
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> Result<Box<dyn Iterator<Item = Model>>> {
        if let Some(related_to) = related_to {
            let prefix = Self::edge_prefix(list_of, related_to);
            let mut models: Vec<Model> = vec![];
            let like_prefix = format!("{}%", prefix);
            for related_key in self._list(&like_prefix)? {
                let value = self._get(&String::from_utf8(related_key)?)?;
                if value.is_none() {
                    continue;
                }
                let model: Model = bincode::deserialize(&value.unwrap()).unwrap();
                models.push(model);
            }
            Ok(Box::new(models.into_iter()))
        } else {
            let prefix = Self::node_prefix(list_of);
            let mut models: Vec<Model> = vec![];
            let like_prefix = format!("{}%", prefix);
            for value in self._list(&like_prefix)? {
                let model: Model = bincode::deserialize(&value).unwrap();
                models.push(model);
            }
            Ok(Box::new(models.into_iter()))
        }
    }

    fn reset(&self) -> Result<()> {
        self.con.execute("DELETE FROM kv")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::model::{Entity, ReleaseGroup};

    use super::*;

    #[test]
    fn basics() {
        let db = SqliteDb::new(":memory:");
        let a1 = db.insert(&Artist::default().model()).unwrap();
        let a2 = db.insert(&Artist::default().model()).unwrap();
        let r1 = db.insert(&Release::default().model()).unwrap();
        db.link(&a2, &r1).unwrap();
        for artist in db.list(&Artist::default().model(), &None).unwrap() {
            println!("artist");
            for release in db.list(&Release::default().model(), &Some(artist)).unwrap() {
                println!("  release");
                for track in db.list(&Track::default().model(), &Some(release)).unwrap() {
                    println!("    track");
                }
            }    
        }
    }

    #[test]
    fn more() {
        let db = SqliteDb::new(":memory:");
        for i in 0..10000 {
            let artist = Artist {
                name: Some(format!("{} {} {} {} {}", i, i, i, i, i)),
                ..Default::default()
            };
            let artist = db.insert(&artist.model()).unwrap();
            let release = db.insert(&Release::default().model()).unwrap();
            db.link(&artist, &release).unwrap();

            let release = db.insert(&Release::default().model()).unwrap();
            db.link(&artist, &release).unwrap();

            let release = db.insert(&Release::default().model()).unwrap();
            db.link(&artist, &release).unwrap();
        }
        let now = Instant::now();
        let artist = db.list(&Artist::default().model(), &None).unwrap()
            .map(Into::<Artist>::into)
            .find(|artist| {
                artist.name == Some("357 357 357 357 357".to_string())
            });
        println!("found {:?} in {}", artist, Instant::now().duration_since(now).as_millis());
    }
}
