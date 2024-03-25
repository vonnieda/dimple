use std::{collections::HashMap, rc::Rc, sync::Mutex};

use anyhow::Result;

use uuid::Uuid;

use crate::model::Model;

use super::Db;

#[derive(Default)]
pub struct MemoryDb {
    map: Rc<Mutex<HashMap<String, String>>>,
}

impl MemoryDb {
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

impl Db for MemoryDb {
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
        self.map.lock().unwrap().insert(key, json);
        Ok(model)
    }

    fn get(&self, model: &Model) -> Result<Option<Model>> {
        let key = Self::node_key(model);
        let map = self.map.lock().unwrap();
        let json = map.get(&key);
        if json.is_none() {
            return Ok(None)
        }
        let model = serde_json::from_str(json.unwrap())?;
        Ok(Some(model))
    }

    fn link(&self, model: &Model, related_to: &Model) -> Result<()> {
        let mut map = self.map.lock().unwrap();

        // related_to -> model
        let key = Self::edge_key(model, related_to);
        let related_key = Self::node_key(model);
        map.insert(key, related_key);

        // model -> related_to
        // TODO not sure if I want this to be bi-dir by default or not
        let key = Self::edge_key(related_to, model);
        let related_key = Self::node_key(related_to);
        map.insert(key, related_key);

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
            let map = self.map.lock().unwrap();
            for key in map.keys() {
                if key.starts_with(&prefix) {
                    let related_key = map.get(key).unwrap();
                    let json = map.get(related_key).unwrap();
                    let model = serde_json::from_str(json)?;
                    models.push(model);
                }
            }
            Ok(Box::new(models.into_iter()))
        }
        else {
            let prefix = Self::node_prefix(list_of);
            let mut models: Vec<Model> = vec![];
            let map = self.map.lock().unwrap();
            for key in map.keys() {
                if key.starts_with(&prefix) {
                    let json = map.get(key).unwrap();
                    let model = serde_json::from_str(json)?;
                    models.push(model);
                }
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
