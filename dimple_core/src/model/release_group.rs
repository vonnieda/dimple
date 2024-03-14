use std::collections::HashSet;


use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;
use crate::model::Release;
use crate::model::Entity;
use crate::model::Artist;
use crate::model::Genre;


use crate::collection::Collection;

use super::Entities;

// https://musicbrainz.org/doc/ReleaseGroup
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct ReleaseGroup {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub first_release_date: Option<String>,
    pub primary_type: Option<String>,
}

impl ReleaseGroup {
    pub fn entity(&self) -> Entities {
        Entities::ReleaseGroup(self.clone())
    }

    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = ReleaseGroup {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::ReleaseGroup(r)) => Some(r),
            _ => todo!()
        }
    }

    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = ReleaseGroup>> {
        let iter = col.list(&ReleaseGroup::default().entity(), None)
            .map(|m| match m {
                Entities::ReleaseGroup(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn releases(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
        let iter = lib.list(&Release::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Release(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn genres(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Genre>> {
        let iter = lib.list(&Genre::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Genre(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn artists(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.list(&Artist::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Artist(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}

impl Entity for ReleaseGroup {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::ReleaseGroup(self.clone())
    }    
}
