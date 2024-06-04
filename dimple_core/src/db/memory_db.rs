use std::{collections::HashMap, sync::{Arc, RwLock}};

use anyhow::Result;

use uuid::Uuid;

use crate::model::{Artist, Entity, Genre, Model, Release, Track};

use super::Db;

#[derive(Default)]
pub struct MemoryDb {
    nodes: Arc<RwLock<HashMap<String, Model>>>,
    edges: Arc<RwLock<HashMap<String, String>>>,
}

impl MemoryDb {
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

impl Db for MemoryDb {
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
        self.nodes.write().unwrap().insert(key, model.clone());
        Ok(model)
    }

    fn get(&self, model: &Model) -> Result<Option<Model>> {
        let key = Self::node_key(model);
        Ok(self.nodes.read().unwrap().get(&key).cloned())
    }

    fn link(&self, model: &Model, related_to: &Model) -> Result<()> {
        // related_to -> model
        let key = Self::edge_key(model, related_to);
        let related_key = Self::node_key(model);
        self.edges.write().unwrap().insert(key, related_key);

        // model -> related_to
        let key = Self::edge_key(related_to, model);
        let related_key = Self::node_key(related_to);
        self.edges.write().unwrap().insert(key, related_key);

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
            for related_key in self.edges.read().unwrap().keys() {
                if related_key.starts_with(&prefix) {
                    if let Some(key) = self.edges.read().unwrap().get(related_key) {
                        // TODO might be a deadlock, maybe need to lock both
                        if let Some(model) = self.nodes.read().unwrap().get(key) {
                            models.push(model.clone())
                        }
                    }
                }
            }
            Ok(Box::new(models.into_iter()))
        } else {
            let prefix = Self::node_prefix(list_of);
            let mut models: Vec<Model> = vec![];
            // TODO change to an if let so we only lock once
            for key in self.nodes.read().unwrap().keys() {
                if key.starts_with(&prefix) {
                    if let Some(model) = self.nodes.read().unwrap().get(key) {
                        models.push(model.clone());
                    }
                }
            }
            Ok(Box::new(models.into_iter()))
        }
    }

    fn reset(&self) -> Result<()> {
        self.nodes.write().unwrap().clear();
        self.edges.write().unwrap().clear();
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
        let db = MemoryDb::default();
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
        let db = MemoryDb::default();
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
        // let artist = db.list(&Artist::default().model(), &None).unwrap()
        //     .map(Into::<Artist>::into)
        //     .find(|artist| {
        //         artist.name == Some("357 357 357 357 357".to_string())
        //     });
        // println!("found {:?} in {}", artist, Instant::now().duration_since(now).as_millis());
    }
}
