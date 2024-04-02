use anyhow::Result;

use uuid::Uuid;

use crate::model::{Model};

use super::Db;

use sqlite::{Connection, ConnectionThreadSafe, State};

pub struct SqliteDb {
    con: ConnectionThreadSafe,
}

impl SqliteDb {
    pub fn new(path: &str) -> Self {
        let con = Connection::open_thread_safe(path).unwrap();
        con.execute("CREATE TABLE IF NOT EXISTS kv (key TEXT NOT NULL PRIMARY KEY, value TEXT NOT NULL)").unwrap();
        Self {
            con,
        }
    }

    fn _get(&self, key: &str) -> Result<Option<String>> {
        let mut statement = self.con.prepare("SELECT value FROM kv WHERE key = :key")?;
        statement.bind((":key", key))?;
        if let Ok(State::Row) = statement.next() {
            let value = statement.read::<String, _>("value")?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    fn _insert(&self, key: &str, value: &str) -> Result<()> {
        let mut statement = self.con.prepare("REPLACE INTO kv (key, value) VALUES (:key, :value)")?;
        statement.bind((":key", key))?;
        statement.bind((":value", value))?;
        statement.next()?;
        Ok(())
    }

    fn _list(&self, prefix: &str) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut statement = self.con.prepare("SELECT value FROM kv WHERE key LIKE :prefix")?;
        statement.bind((":prefix", prefix))?;
        let mut results: Vec<String> = vec![];
        while let Ok(State::Row) = statement.next() {
            let value = statement.read::<String, _>("value")?;
            results.push(value);
        }
        Ok(Box::new(results.into_iter()))
    }

    fn node_key(model: &Model) -> String {
        // type:key
        format!("node:{}:{}", model.entity_name(), model.key().unwrap())
    }

    fn node_prefix(model: &Model) -> String {
        // type:
        format!("node:{}:", model.entity_name())
    }

    fn edge_key(model: &Model, related_to: &Model) -> String {
        // edge_key(release, artist) -> atype:btype:akey:bkey
        format!(
            "edge:{}:{}:{}:{}",
            related_to.entity_name(),
            related_to.key().unwrap(),
            model.entity_name(),
            model.key().unwrap(),
        )
    }

    fn edge_prefix(model: &Model, related_to: &Model) -> String {
        // edge_prefix(release, artist) -> atype:btype:akey:
        format!(
            "edge:{}:{}:{}:",
            related_to.entity_name(),
            related_to.key().unwrap(),
            model.entity_name(),
        )
    }
}

impl Db for SqliteDb {
    fn insert(&self, model: &Model) -> Result<Model> {
        let model = match model.key() {
            Some(_) => model.clone(),
            None => {
                let mut model = model.clone();
                model.set_key(Some(Uuid::new_v4().to_string()));
                model
            }
        };
        let key = Self::node_key(&model);
        let json = serde_json::to_string(&model)?;
        self._insert(&key, &json)?;
        Ok(model)
    }

    fn get(&self, model: &Model) -> Result<Option<Model>> {
        let key = Self::node_key(model);
        let json = self._get(&key)?;
        if json.is_none() {
            return Ok(None);
        }
        let model = serde_json::from_str(&json.unwrap())?;
        Ok(Some(model))
    }

    fn link(&self, model: &Model, related_to: &Model) -> Result<()> {
        // related_to -> model
        let key = Self::edge_key(model, related_to);
        let related_key = Self::node_key(model);
        self._insert(&key, &related_key)?;

        // model -> related_to
        // TODO not sure if I want this to be bi-dir by default or not
        let key = Self::edge_key(related_to, model);
        let related_key = Self::node_key(related_to);
        self._insert(&key, &related_key)?;

        Ok(())
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: Option<&Model>,
    ) -> Result<Box<dyn Iterator<Item = Model>>> {
        if let Some(related_to) = related_to {
            let prefix = Self::edge_prefix(list_of, related_to);
            let mut models: Vec<Model> = vec![];
            let like_prefix = format!("{}%", prefix);
            for related_key in self._list(&like_prefix)? {
                let json = self._get(&related_key)?;
                if json.is_none() {
                    continue;
                }
                let model = serde_json::from_str(&json.unwrap())?;
                models.push(model);
            }
            Ok(Box::new(models.into_iter()))
        } else {
            let prefix = Self::node_prefix(list_of);
            let mut models: Vec<Model> = vec![];
            let like_prefix = format!("{}%", prefix);
            for json in self._list(&like_prefix)? {
                let model = serde_json::from_str(&json)?;
                models.push(model);
            }
            Ok(Box::new(models.into_iter()))
        }
    }
}

trait Entity {
    fn entity_name(&self) -> String;
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
}

impl Entity for Model {
    fn entity_name(&self) -> String {
        match self {
            Model::Artist(_) => "Artist".to_string(),
            Model::Genre(_) => "ReleaseGroup".to_string(),
            Model::MediaFile(_) => "ReleaseGroup".to_string(),
            Model::Medium(_) => "ReleaseGroup".to_string(),
            Model::Recording(_) => "ReleaseGroup".to_string(),
            Model::RecordingSource(_) => "ReleaseGroup".to_string(),
            Model::Release(_) => "Release".to_string(),
            Model::ReleaseGroup(_) => "ReleaseGroup".to_string(),
            Model::Track(_) => "ReleaseGroup".to_string(),
        }
    }

    fn key(&self) -> Option<String> {
        match self {
            Model::Artist(value) => value.key.clone(),
            Model::Genre(value) => value.key.clone(),
            Model::MediaFile(value) => value.key.clone(),
            Model::Medium(value) => value.key.clone(),
            Model::Recording(value) => value.key.clone(),
            Model::RecordingSource(value) => value.key.clone(),
            Model::Release(value) => value.key.clone(),
            Model::ReleaseGroup(value) => value.key.clone(),
            Model::Track(value) => value.key.clone(),
        }
    }

    fn set_key(&mut self, key: Option<String>) {
        match self {
            Model::Artist(value) => value.key = key,
            Model::Genre(value) => value.key = key,
            Model::MediaFile(value) => value.key = key,
            Model::Medium(value) => value.key = key,
            Model::Recording(value) => value.key = key,
            Model::RecordingSource(value) => value.key = key,
            Model::Release(value) => value.key = key,
            Model::ReleaseGroup(value) => value.key = key,
            Model::Track(value) => value.key = key,
        }
    }
}
