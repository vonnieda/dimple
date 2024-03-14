use std::collections::HashSet;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;
use crate::model::Release;
use crate::model::Entity;
use crate::model::ReleaseGroup;
use crate::model::Artist;
use crate::model::Recording;
use crate::collection::Collection;

use super::Entities;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Genre {
    pub key: Option<String>,
    pub name: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,
}

// impl Genre {
    // pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
    //     let ent = Self {
    //         key: Some(key.to_string()),
    //         ..Default::default()
    //     }.entity();
    //     match lib.fetch(&ent) {
    //         Some(Entities::Genre(g)) => Some(g),
    //         _ => todo!()
    //     }
    // }

    // pub fn entity(&self) -> Entities {
    //     Entities::Genre(self.clone())
    // }

    // pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
    //     let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
    //     let iter = iter.map(|r| match r {
    //         Entities::Recording(r) => r,
    //         _ => panic!(),
    //     }); 
    //     Box::new(iter)    
    // }

    // pub fn releases(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
    //     let iter = lib.list(&Release::default().entity(), Some(&self.entity()));
    //     let iter = iter.map(|r| match r {
    //         Entities::Release(r) => r,
    //         _ => panic!(),
    //     }); 
    //     Box::new(iter)    
    // }

    // pub fn artists(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
    //     let iter = lib.list(&Artist::default().entity(), Some(&self.entity()));
    //     let iter = iter.map(|r| match r {
    //         Entities::Artist(r) => r,
    //         _ => panic!(),
    //     }); 
    //     Box::new(iter)    
    // }
// }

impl Entity for Genre {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::Genre(self.clone())
    }    
}
