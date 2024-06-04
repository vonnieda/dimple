use std::{collections::HashSet, fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, ArtistCredit, Entity, Genre, Medium, Model, Release, ReleaseGroup, Track}
};

use anyhow::Result;

use crate::{merge::Merge, plugin::{NetworkMode, Plugin}};

use rayon::prelude::{*};

#[derive(Clone)]
pub struct Librarian {
    db: Arc<Box<dyn Db>>,
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
    network_mode: Arc<Mutex<NetworkMode>>,
}

impl Librarian {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).unwrap();
        let db_path = Path::new(path).join("dimple.db");
        let librarian = Self {
            db: Arc::new(Box::new(SqliteDb::new(db_path.to_str().unwrap()))),
            plugins: Default::default(),
            network_mode: Arc::new(Mutex::new(NetworkMode::Online)),
        };
        librarian
    }

    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    }

    pub fn network_mode(&self) -> NetworkMode {
        self.network_mode.lock().unwrap().clone()
    }

    pub fn set_network_mode(&self, network_mode: &NetworkMode) {
        *self.network_mode.lock().unwrap() = network_mode.clone();
    }

    // TODO Still struggling mightily with this, but I think it's worth the
    // effort to just go down this path, even if it gets thrown away, just to
    // get it working.
    fn merge(&self, model: &Model) -> Option<Model> {
        match model {
            // TODO I think I can move this logic into db_merge_model, and specifically
            // panic when asked to merge something without enough context. 
            // Like an Media without a Release or whatever.
            Model::Artist(artist) => self.merge_artist(artist),
            Model::Release(release) => self.merge_release(release),
            Model::ReleaseGroup(release_group) => self.merge_release_group(release_group),
            _ => todo!(),
        }
    }

    fn merge_artist(&self, artist: &Artist) -> Option<Model> {
        // TODO This db thing is a footgun, because if I accidentally pass
        // Librarian it will take it, and use the Librarian Db interface
        // which does remotes. Maybe that means Librarian should not be
        // Db, to be safe.
        let db: &dyn Db = self.db.as_ref().as_ref();
        let artist: Artist = Self::db_merge_model(db, &artist.model(), &None)?.into();
        for genre in &artist.genres {
            let genre = self.merge_genre(genre);
            Self::lazy_link(db, &genre, &Some(artist.model()))
        }
        Some(artist.model())
    }

    fn merge_release_group(&self, release_group: &ReleaseGroup) -> Option<Model> {
        let db: &dyn Db = self.db.as_ref().as_ref();

        let release_group: ReleaseGroup = 
            Self::db_merge_model(db, &release_group.model(), &None)?.into();

        for genre in &release_group.genres {
            let genre = self.merge_genre(genre);
            Self::lazy_link(db, &genre, &Some(release_group.model()))
        }

        // TODO temporary bypass artist credit for artist to get some testing
        // done
        for artist_credit in &release_group.artist_credits {
            let artist = self.merge_artist(&artist_credit.artist);
            Self::lazy_link(db, &artist, &Some(release_group.model()))
        }

        Some(release_group.model())
    }

    /// get, merge, update the release properties
    /// for each genre
    ///     merge(genre, release)
    ///     link(release, genre)
    /// for each artist_credit
    ///     merge(artist_credit)
    ///     link(release, artist_credit)
    ///     link(release, artist)
    /// for each medium
    ///     for each track
    ///         merge(release, medium, track)
    fn merge_release(&self, release: &Release) -> Option<Model> {
        let db: &dyn Db = self.db.as_ref().as_ref();
        Self::db_merge_model(db, &release.model(), &None)
    }

    fn merge_genre(&self, genre: &Genre) -> Option<Model> {
        let db: &dyn Db = self.db.as_ref().as_ref();
        Self::db_merge_model(db, &genre.model(), &None)
    }

    fn db_merge_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
        // TODO does this need to be merging parent as well?

        // find a matching model to the specified, merge, save
        let matching = Self::find_matching_model(db, model, parent);
        if let Some(matching) = matching {
            let merged = Model::merge(model.clone(), matching);
            return Some(db.insert(&merged).unwrap())
        }
        // if not, insert the new one and link it to the parent
        else {
            if Self::model_valid(model) {
                let model = Some(db.insert(model).unwrap());
                Self::lazy_link(db, &model, parent);
                return model
            }
        }
        None
    }

    /// Links the two Models if they are both Some. Reduces boilerplate.
    fn lazy_link(db: &dyn Db, l: &Option<Model>, r: &Option<Model>) {
        if l.is_some() && r.is_some() {
            db.link(&l.clone().unwrap(), &r.clone().unwrap()).unwrap()
        }
    }

    fn find_matching_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
        match model {
            Model::ReleaseGroup(release_group) => Self::find_release_group(db, release_group),
            _ => db.list(&model, parent).unwrap().find(|model_opt| Self::compare_models(&model, model_opt))
        }
    }

    fn find_release_group(db: &dyn Db, release_group: &ReleaseGroup) -> Option<Model> {
        // find by key
        if let Some(_key) = &release_group.key {
            return db.get(&release_group.model()).unwrap()
        }

        // find by known id
        let matched = db.list(&release_group.model(), &None).unwrap()
            .map(Into::<ReleaseGroup>::into)
            .find(|opt| {
                Self::is_some_and_equal(&release_group.known_ids.musicbrainz_id, &opt.known_ids.musicbrainz_id)
            });
        if let Some(matched) = matched {
            return Some(matched.model())
        }

        // find by artist + title
        None
    }

    fn is_some_and_equal(l: &Option<String>, r: &Option<String>) -> bool {
        l.is_some() && l == r
    }

    fn compare_models(l: &Model, r: &Model) -> bool {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => {
                (l.name.is_some() && l.name == r.name && l.disambiguation == r.disambiguation)
                || (l.known_ids.musicbrainz_id.is_some() && l.known_ids.musicbrainz_id == r.known_ids.musicbrainz_id)
            },
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => {
                (l.title.is_some() && l.title == r.title)
                || (l.known_ids.musicbrainz_id.is_some() && l.known_ids.musicbrainz_id == r.known_ids.musicbrainz_id)
            },
            (Model::Release(l), Model::Release(r)) => {
                l.title.is_some() && l.title == r.title
            },
            (Model::Medium(l), Model::Medium(r)) => {
                l.position == r.position
            },
            (Model::Track(l), Model::Track(r)) => {
                l.title.is_some() && l.title == r.title
            },
            (Model::Genre(l), Model::Genre(r)) => {
                l.name.is_some() && l.name == r.name
            },
            (Model::ArtistCredit(l), Model::ArtistCredit(r)) => {
                l.name.is_some() && l.name == r.name
            },
            _ => todo!()
        }
    }

    fn model_valid(model: &Model) -> bool {
        match model {
            Model::Artist(a) => a.name.is_some() || a.known_ids.musicbrainz_id.is_some(),
            Model::Genre(g) => g.name.is_some(),
            Model::Medium(_m) => true,
            Model::Release(r) => r.title.is_some(),
            Model::ReleaseGroup(rg) => rg.title.is_some(),
            Model::Track(t) => t.title.is_some(),
            _ => todo!()
        }
    }

    // fn local_library_search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
    //     // const MAX_RESULTS_PER_TYPE: usize = 10;
    //     // TODO sort by score
    //     // TODO other entities
    //     let pattern = query.to_string();
    //     let matcher = SkimMatcherV2::default();
    //     let artists = Artist::list(&self.local_library)
    //         .filter(move |a| matcher.fuzzy_match(&a.name.clone().unwrap_or_default(), &pattern).is_some())
    //         .map(Entities::Artist);
    //         // .take(MAX_RESULTS_PER_TYPE);
    //     let pattern = query.to_string();
    //     let matcher = SkimMatcherV2::default();
    //     let release_groups = ReleaseGroup::list(&self.local_library)
    //         .filter(move |a| matcher.fuzzy_match(&a.title.clone().unwrap_or_default(), &pattern).is_some())
    //         .map(Entities::ReleaseGroup);
    //         // .take(MAX_RESULTS_PER_TYPE);
    //     Box::new(artists.chain(release_groups))
    // }

    pub fn search(&self, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.search(query, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    self.merge(&result);
                }
            }
        }

        // TODO fts lol
        // self.db.search(query)
        let results = self.db.list(&Artist::default().model(), &None)?
            .chain(self.db.list(&Release::default().model(), &None)?)
            .chain(self.db.list(&ReleaseGroup::default().model(), &None)?)
            .chain(self.db.list(&Genre::default().model(), &None)?)
            .chain(self.db.list(&Track::default().model(), &None)?);
            
        Ok(Box::new(results))
    }
}

impl Db for Librarian {
    fn insert(&self, model: &dimple_core::model::Model) -> Result<dimple_core::model::Model> {
        self.db.insert(model)
    }

    /// Get the specified model by a unique identifier. This will either be by
    /// key for stored data, or via a plugin defined key for plugins. The data
    /// from the local storage, if any, along with data returned by plugins
    /// is merged together and then merged into the database before being
    /// returned. 
    /// 
    /// If there are no results from storage or a plugin, returns Ok(None)
    fn get(&self, model: &Model) -> Result<Option<Model>> {
        let mut model = model.clone();

        if let Ok(Some(db_model)) = self.db.get(&model) {
            model = Model::merge(model, db_model);
        }

        let mut finished_plugins = HashSet::<String>::new();
        for plugin in self.plugins.read().unwrap().iter() {
            if let Ok(Some(plugin_model)) = plugin.get(&model, self.network_mode()) {
                model = Model::merge(model, plugin_model);
                finished_plugins.insert(plugin.name());
            }
        }

        for plugin in self.plugins.read().unwrap().iter() {
            if finished_plugins.contains(&plugin.name()) {
                continue;
            }
            if let Ok(Some(plugin_model)) = plugin.get(&model, self.network_mode()) {
                model = Model::merge(model, plugin_model);
            }
        }

        Ok(self.merge(&model))
    }

    fn link(&self, model: &dimple_core::model::Model, related_to: &dimple_core::model::Model) -> Result<()> {
        self.db.link(model, related_to)
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
    ) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
        let db: &dyn Db = self.db.as_ref().as_ref();
        for plugin in self.plugins.read().unwrap().iter() {
            let results = plugin.list(list_of, related_to, self.network_mode());
            if let Ok(results) = results {
                for result in results {
                    // TODO noting the use of db_merge_model here vs. self.merge in search
                    // because this asks for objects by relationship and thus needs to be
                    // merged with that same relationship.
                    Self::db_merge_model(db, &result, related_to);
                }
            }
        }

        self.db.list(list_of, related_to)
    }
    
    fn reset(&self) -> Result<()> {
        self.db.reset()
    }
}

