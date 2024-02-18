use std::borrow::Borrow;

use dimple_core::{collection::Collection, image_cache::ImageCache};
use dimple_core::model::Entities;

use image::{DynamicImage, EncodableLayout};

use sled::{Db, Tree};

use uuid::Uuid;

#[derive(Debug)]

/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes. Object are serialized as JSON,
/// media is stored raw.
/// 
/// Somewhere along the way this became a graph overlaid on a key/value store.
/// Probably this can use any KV store now. 
pub struct SledLibrary {
    path: String,
    db: Db,
    pub images: ImageCache,
}

impl SledLibrary {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        let images = ImageCache::new(db.open_tree("images").unwrap());
        Self { 
            path: path.to_string(),
            db,
            images,
        }
    }

    pub fn get(&self, model: &Entities) -> Option<Entities> {
        model.key().as_deref()?;
        let key = Self::vertex_key(model);
        let value = self.db.get(key).ok()??;
        let bytes = value.as_bytes();
        let json = String::from_utf8(bytes.into()).ok()?;
        serde_json::from_str(&json).ok()?
    }

    pub fn list_(&self, of_type: &Entities) -> Box<dyn Iterator<Item = Entities>> {
        let prefix = Self::vertex_prefix(of_type);
        let iter = self.db.scan_prefix(prefix).map(|t| {
            let (k, v) = t.unwrap();
            serde_json::from_slice(v.borrow()).unwrap()
        });
        Box::new(iter)
    }

    pub fn links(&self, a: &Entities, b: &Entities, relation: &str) -> Box<dyn Iterator<Item = Entities>> {
        let prefix = Self::edge_prefix(a, b, relation);
        let b = b.clone();
        let recs: Vec<_> = self.db.scan_prefix(prefix).filter_map(move |t| {
            let (k, v) = t.unwrap();
            let key = String::from_utf8(v.to_vec()).unwrap();
            let mut b = b.clone();
            b.set_key(Some(key));
            self.get(&b)
        }).collect();
        Box::new(recs.into_iter())
    }

    pub fn clear(&self) {
        let _ = self.db.clear();
    }

    pub fn set(&self, model: &Entities) -> anyhow::Result<Entities> {
        let model = match model.key() {
            Some(_) => model.clone(),
            None => {
                let mut model = model.clone();
                model.set_key(Some(Uuid::new_v4().to_string()));
                model
            }
        };
        let json = serde_json::to_string(&model)?;
        let key = Self::vertex_key(&model);
        let _ = self.db.insert(key, &*json)?;
        Ok(model)
    }

    pub fn link(&self, a: &Entities, b: &Entities, relation: &str) -> anyhow::Result<()> {
        let key_a = a.key().expect("a.key must be Some");
        let key_b = b.key().expect("b.key must be Some");
        let key = Self::edge_key(a, b, relation);        
        let _ = self.db.insert(key, key_b.as_bytes())?;
        let key = Self::edge_key(b, a, relation);        
        let _ = self.db.insert(key, key_a.as_bytes())?;
        Ok(())
    }

    fn vertex_key(model: &Entities) -> String {
        // type:key
        format!("{}:{}", model.type_name(), model.key().unwrap())
    }

    fn vertex_prefix(model: &Entities) -> String {
        // type:
        format!("{}:", model.type_name())
    }

    fn edge_key(a: &Entities, b: &Entities, relation: &str) -> String {
        // edge_key(release, artist, "artist_credit") -> relation:atype:btype:akey:bkey
        format!("{}:{}:{}:{}:{}",
            relation, 
            a.type_name(), 
            b.type_name(), 
            a.key().unwrap(),
            b.key().unwrap())
    }

    fn edge_prefix(a: &Entities, b: &Entities, relation: &str) -> String {
        // edge_prefix(release, artist, "artist_credit") -> relation:atype:btype:akey:
        format!("{}:{}:{}:{}",
            relation, 
            a.type_name(), 
            b.type_name(), 
            a.key().unwrap())
    }

    pub fn set_image(&self, for_entity: &Entities, image: &DynamicImage) {
        self.images.insert(&for_entity.key().unwrap(), image);
    }
}

impl Collection for SledLibrary {
    fn name(&self) -> String {
        format!("SledLibrary({})", self.path)
    }

    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = Entities>> {
        todo!();
    }    

    fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
        if let Some(related_to) = related_to {
            // TODO this feels wrong
            self.links(related_to, of_type, "by")
        }
        else {
            self.list_(of_type)
        }
    }

    fn fetch(&self, entity: &Entities) -> Option<Entities> {
        self.get(entity)
    }

    fn image(&self, entity: &Entities) -> Option<DynamicImage> {
        self.images.get_original(&entity.key().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use dimple_core::model::{Artist, Release};

    use super::*;

    #[test]
    fn basics() {
        let lib = SledLibrary::new(".sled");
        lib.clear();
        
        let metallicurds = lib.set(&Artist {
            name: Some("Metallicurds".to_string()),
            ..Default::default()
        }.entity()).unwrap();

        let and_fresh_curds = lib.set(&Release {
            title: Some("...And Fresh Curds For All".to_string()),
            ..Default::default()
        }.entity()).unwrap();
        
        let master_of_pasteurization = lib.set(&Release {
            title: Some("Master of Pasteurization".to_string()),
            ..Default::default()
        }.entity()).unwrap();
        
        let ride_the_milkfat = lib.set(&Release {
            title: Some("Ride the Milkfat".to_string()),
            ..Default::default()
        }.entity()).unwrap();
        
        lib.link(&metallicurds, &and_fresh_curds, "artist_credit").unwrap();
        lib.link(&metallicurds, &master_of_pasteurization, "artist_credit").unwrap();
        lib.link(&metallicurds, &ride_the_milkfat, "artist_credit").unwrap();

        let moo_cheese = lib.set(&Artist {
            name: Some("Moo Cheese".to_string()),
            ..Default::default()
        }.entity()).unwrap();

        let transfonduer = lib.set(&Release {
            title: Some("Transfonduer".to_string()),
            ..Default::default()
        }.entity()).unwrap();

        lib.link(&moo_cheese, &transfonduer, "artist_credit").unwrap();

        let mumu = lib.set(&Release {
            title: Some("Mumu".to_string()),
            ..Default::default()
        }.entity()).unwrap();

        lib.link(&metallicurds, &mumu, "artist_credit").unwrap();
        lib.link(&moo_cheese, &mumu, "artist_credit").unwrap();

        let artists = lib.list(&Artist::default().entity(), None);
        // for artist in artists {
        //     let artist: Artist = (&artist).into();
        //     println!("{}",artist.name.clone().unwrap());
        //     let releases = lib.links(&artist.entity(), &Release::default().entity(), "artist_credit");
        //     for release in releases {
        //         let release: Release = (&release).into();
        //         println!("    {}", release.title.clone().unwrap());
        //         let artists = lib.links(&release.entity(), &Artist::default().entity(), "artist_credit");
        //         for artist in artists {
        //             let artist: Artist = (&artist).into();
        //             println!("        {}",artist.name.clone().unwrap());
        //         }
        //     }
        // }
    }
}
