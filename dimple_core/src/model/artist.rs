use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::collection::Collection;

use crate::model::KnownId;
use crate::model::Entity;
use crate::model::Release;
use crate::model::ReleaseGroup;
use crate::model::Recording;
use crate::model::Genre;

use super::Entities;

// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub country: Option<String>,
}

impl Artist {
    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = col.list(&Artist::default().entity(), None)
            .map(|m| match m {
                Entities::Artist(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = Artist {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::Artist(a)) => Some(a),
            _ => todo!()
        }
    }

    pub fn entity(&self) -> Entities {
        Entities::Artist(self.clone())
    }

    pub fn search(query: &str, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.search(query)
            .filter_map(|m| match m {
                Entities::Artist(a) => Some(a),
                _ => None,
            });
        Box::new(iter)
    }

    pub fn release_groups(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = ReleaseGroup>> {
        let iter = lib.list(&ReleaseGroup::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::ReleaseGroup(r) => r,
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

    pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Recording(r) => r,
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
}

impl Entity for Artist {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::Artist(self.clone())
    }    
}
