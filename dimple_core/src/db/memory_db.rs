use std::{collections::HashMap, sync::{Arc, RwLock}};

use anyhow::Result;

use uuid::Uuid;

use crate::model::{Artist, Genre, Model, Release, Track};

use super::Db;

#[derive(Default)]
pub struct MemoryDb {
    nodes: Arc<RwLock<HashMap<String, Model>>>,
    edges: Arc<RwLock<HashMap<String, String>>>,
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
        related_to: Option<&Model>,
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
    
    fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = Model>>> {
        let iter = self.list(&crate::model::Entity::model(&Artist::default()), None).unwrap().take(10)
            .chain(self.list(&crate::model::Entity::model(&Release::default()), None).unwrap().take(10))
            .chain(self.list(&crate::model::Entity::model(&Genre::default()), None).unwrap().take(10));
        Ok(Box::new(iter))
    }
}

// TODO now that we have Entity in core this can all go away
trait Entity {
    fn entity_name(&self) -> String;
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
}

impl Entity for Model {
    fn entity_name(&self) -> String {
        match self {
            Model::Artist(_) => "Artist".to_string(),
            Model::ArtistCredit(_) => "ArtistCredit".to_string(),
            Model::Blob(_) => "Blob".to_string(),
            Model::Genre(_) => "Genre".to_string(),
            Model::Medium(_) => "Medium".to_string(),
            Model::Recording(_) => "Recording".to_string(),
            Model::RecordingSource(_) => "RecordingSource".to_string(),
            Model::Release(_) => "Release".to_string(),
            Model::ReleaseGroup(_) => "ReleaseGroup".to_string(),
            Model::Track(_) => "Track".to_string(),
            Model::Picture(_) => "Picture".to_string(),
            Model::Playlist(_) => "Playlist".to_string(),
            Model::PlaylistItem(_) => "PlaylistItem".to_string(),
        }
    }

    fn key(&self) -> Option<String> {
        match self {
            Model::Picture(value) => value.key.clone(),
            Model::Artist(value) => value.key.clone(),
            Model::ArtistCredit(value) => value.key.clone(),
            Model::Blob(value) => value.key.clone(),
            Model::Genre(value) => value.key.clone(),
            Model::Medium(value) => value.key.clone(),
            Model::Recording(value) => value.key.clone(),
            Model::RecordingSource(value) => value.key.clone(),
            Model::Release(value) => value.key.clone(),
            Model::ReleaseGroup(value) => value.key.clone(),
            Model::Track(value) => value.key.clone(),
            Model::Playlist(value) => value.key.clone(),
            Model::PlaylistItem(value) => value.key.clone(),
        }
    }

    fn set_key(&mut self, key: Option<String>) {
        match self {
            Model::Picture(value) => value.key = key,
            Model::Artist(value) => value.key = key,
            Model::ArtistCredit(value) => value.key = key,
            Model::Blob(value) => value.key = key,
            Model::Genre(value) => value.key = key,
            Model::Medium(value) => value.key = key,
            Model::Recording(value) => value.key = key,
            Model::RecordingSource(value) => value.key = key,
            Model::Release(value) => value.key = key,
            Model::ReleaseGroup(value) => value.key = key,
            Model::Track(value) => value.key = key,
            Model::Playlist(value) => value.key = key,
            Model::PlaylistItem(value) => value.key = key,
        }
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
        for artist in db.list(&Artist::default().model(), None).unwrap() {
            println!("artist");
            for release in db.list(&Release::default().model(), Some(&artist)).unwrap() {
                println!("  release");
                for track in db.list(&Track::default().model(), Some(&release)).unwrap() {
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
        let artist = db.list(&Artist::default().model(), None).unwrap()
            .map(Into::<Artist>::into)
            .find(|artist| {
                artist.name == Some("357 357 357 357 357".to_string())
            });
        println!("found {:?} in {}", artist, Instant::now().duration_since(now).as_millis());
    }
}
