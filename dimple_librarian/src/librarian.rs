use std::{fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Model, Release, ReleaseGroup}
};

use anyhow::Result;

use crate::{merge::Merge, plugin::{NetworkMode, Plugin}};

#[derive(Clone)]
pub struct Librarian {
    db: Arc<SqliteDb>,
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
    network_mode: Arc<Mutex<NetworkMode>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).unwrap();
        let db_path = Path::new(path).join("dimple.db");
        let librarian = Self {
            db: Arc::new(SqliteDb::new(db_path.to_str().unwrap())),
            plugins: Default::default(),
            network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
        };
        librarian
    }

    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        plugin.init(self);
        plugin.set_network_mode(&self.network_mode());
        self.plugins.write().unwrap().push(plugin);
    }

    pub fn network_mode(&self) -> NetworkMode {
        self.network_mode.lock().unwrap().clone()
    }

    pub fn set_network_mode(&self, network_mode: &NetworkMode) {
        *self.network_mode.lock().unwrap() = network_mode.clone();
        for plugin in self.plugins.write().unwrap().iter_mut() {
            plugin.set_network_mode(network_mode);
        }
    }


    // Note: This is the wrong direction, but good code. I don't think I
    // can merge a single object at once, without it's linkages and such.

    // fn merge_artist(&self, artist: Artist) -> Option<Model> {
    //     let mut artists: Vec<(Artist, f32)> = self.list(&Artist::default().model(), None)
    //         .unwrap()
    //         .map(Into::<Artist>::into)
    //         .map(|artist_opt| (artist_opt.clone(), Artist::mergability(&artist, &artist_opt)))
    //         .filter(|(_artist_opt, score)| *score >= 1.0)
    //         .collect();
    //     artists.sort_by(|(_artist_l, score_l), (_artist_r, score_r)| score_l.partial_cmp(score_r).unwrap());

    //     if let Some((artist_opt, _score)) = artists.get(0) {
    //         let merged = Artist::merge(artist_opt.clone(), artist.clone());
    //         Some(self.insert(&merged.model()).unwrap().into())
    //     }
    //     else {
    //         Some(self.insert(&artist.model()).unwrap().into())
    //     }
    // }

    // fn merge_release_group(&self, model: ReleaseGroup) -> Option<Model> {
    //     let mut options: Vec<(ReleaseGroup, f32)> = self.list(&model.model(), None)
    //         .unwrap()
    //         .map(Into::<ReleaseGroup>::into)
    //         .map(|option| (option.clone(), ReleaseGroup::mergability(&model, &option)))
    //         .filter(|(_option, score)| *score >= 1.0)
    //         .collect();
    //     options.sort_by(|(_option_l, score_l), (_option_r, score_r)| score_l.partial_cmp(score_r).unwrap());

    //     if let Some((option, _score)) = options.get(0) {
    //         let merged = ReleaseGroup::merge(option.clone(), model.clone());
    //         Some(self.insert(&merged.model()).unwrap().into())
    //     }
    //     else {
    //         if model.title.is_none() {
    //             return None
    //         }
    //         Some(self.insert(&model.model()).unwrap().into())
    //     }
    // }

    // fn merge_release(&self, release: Release) -> Option<Model> {
    //     let mut releases: Vec<(Release, f32)> = self.list(&release.model(), None)
    //         .unwrap()
    //         .map(Into::<Release>::into)
    //         .map(|option| (option.clone(), Release::mergability(&release, &option)))
    //         .filter(|(_option, score)| *score >= 1.0)
    //         .collect();
    //     releases.sort_by(|(_option_l, score_l), (_option_r, score_r)| score_l.partial_cmp(score_r).unwrap());

    //     // Few notes before I take a break:
    //     // A Release can stand on it's own with no artist credits. Maybe we
    //     // don't have the data yet.
    //     // But without an artist credit, it's hard to know if we should merge
    //     // into it. 
    //     // Maybe just have to chill out on this and say if you don't have a
    //     // minimum of artist and album we can't do album, or I guess it's fine
    //     // to say we'll merge into an album that has high mergability as long
    //     // as an artist matches, including empty?
    //     // Or, hell, that's what we have AcoustID for, and other shit. Merge
    //     // just the Recording in and let other plugins help.
        
    //     /// So, I think it's something like merge artist if we have it.
    //     /// Merge album only if we have artist + album.
    //     /// In which case also make Medium and Track.
    //     /// And always merge Recording, maybe with none of the above.
    //     /// 
    //     /// And another option, which I might prefer, is if we don't have
    //     /// at least artist, album, title, we just emit a log message telling
    //     /// you to tag yo shit.

    //     // if let Some((option, _score)) = releases.get(0) {
    //     //     let merged = ReleaseGroup::merge(option.clone(), model.clone());
    //     //     Some(self.insert(&merged.model()).unwrap().into())
    //     // }
    //     // else {
    //     //     if model.title.is_none() {
    //     //         return None
    //     //     }
    //     //     Some(self.insert(&model.model()).unwrap().into())
    //     // }

    //     todo!()
    // }

    // pub fn merge(&self, model: Model) -> Option<Model> {
    //     match model {
    //         Model::Artist(artist) => self.merge_artist(artist),
    //         Model::ReleaseGroup(release_group) => self.merge_release_group(release_group),
    //         Model::Release(release) => self.merge_release(release),
    //         _ => todo!(),
    //     }
    // }
}

impl Db for Librarian {
    fn insert(&self, model: &dimple_core::model::Model) -> Result<dimple_core::model::Model> {
        self.db.insert(model)
    }

    fn get(&self, model: &dimple_core::model::Model) -> Result<Option<dimple_core::model::Model>> {
        self.db.get(model)
    }

    fn link(&self, model: &dimple_core::model::Model, related_to: &dimple_core::model::Model) -> Result<()> {
        self.db.link(model, related_to)
    }

    fn list(
        &self,
        list_of: &dimple_core::model::Model,
        related_to: Option<&dimple_core::model::Model>,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        self.db.list(list_of, related_to)
    }
    
    fn reset(&self) -> Result<()> {
        self.db.reset()
    }
    
    fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        self.db.search(query)
    }
}
