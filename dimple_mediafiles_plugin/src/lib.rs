use dimple_core::model::{Artist, Entity, Genre, Medium, Model, Picture, RecordingSource, Release, ReleaseGroup, Track};
use walkdir::WalkDir;

use std::{collections::HashSet, path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}, thread, time::Instant};

use dimple_librarian::{librarian::Librarian, merge::Merge, plugin::{NetworkMode, Plugin}};

use dimple_core::db::Db;

use crate::media_file::MediaFile;

mod media_file;

#[derive(Clone)]
pub struct MediaFilesPlugin {
    librarian: Arc<Mutex<Option<Librarian>>>,
    directories: Arc<Mutex<HashSet<PathBuf>>>,
    sender: Sender<()>,
}

impl MediaFilesPlugin {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let plugin = Self {
            sender,
            librarian: Default::default(),
            directories: Default::default(),
        };
        {
            let plugin = plugin.clone();
            thread::spawn(move || {
                for _ in receiver.iter() {
                    plugin.scan();
                }
            });
        }
        plugin
    }

    pub fn monitor_directory(&self, path: &PathBuf) {
        self.directories.lock().unwrap().insert(path.to_path_buf());
        // TODO add file system based directory monitoring
        self.rescan()
    }

    /// Triggers a rescan ether now or after the current scan finishes.
    /// TODO interrupt current and restart
    pub fn rescan(&self) {
        if self.librarian.lock().unwrap().is_some() {
            self.sender.send(()).unwrap();
        }
    }

    // TODO query performance
    // TODO recording and rec source
    fn scan(&self) {
        let directories = self.directories.lock().unwrap().clone();
        let db = self.librarian.lock().unwrap().clone();
        if db.is_none() {
            return;
        }
        let db = db.unwrap();
        let now = Instant::now();
        let mut count = 0;
        let mut skipped = 0;
        for dir in directories {
            for dir_entry in WalkDir::new(dir).into_iter() {
                if dir_entry.is_err() { continue }

                let path = dir_entry.unwrap().into_path();
                if !path.is_file() { continue }

                // Find the matching RecordingSource in the Db, if any.
                let source_id = format!("dmfp://{}", path.to_str().unwrap_or_default());
                let rec_source = db.list(&RecordingSource::default().model(), None).unwrap()
                    .map(Into::<RecordingSource>::into)
                    .find(|rec_source| rec_source.source_id == source_id);

                // Compare last modified of the file and the rec source
                // and if the file is older, continue / skip.
                // TODO currently broken cause we're not creating the rec source
                let mtime = path.metadata().unwrap().modified().unwrap();
                if let Some(rec_source) = &rec_source {
                    if let Some(last_modified) = rec_source.last_modified {
                        if last_modified >= mtime {
                            log::debug!("Skipping {:?}, {:?} {:?} is the same or newer than {:?}", 
                                path, 
                                rec_source.key, 
                                last_modified, 
                                mtime);
                            skipped += 1;
                            continue;
                        }
                    }
                }

                // Read the media file.
                let media_file = MediaFile::new(&path);
                if media_file.is_err() { continue }
                let media_file = media_file.unwrap();
            
                // Merge, create, and link objects
                let artist = Self::db_merge_model(&db, &media_file.artist().model(), &None);

                let mut release_group = None;
                if artist.is_some() {
                    release_group = Self::db_merge_model(&db, &media_file.release_group().model(), &artist);
                    Self::lazy_link(&db, &release_group, &artist);
                }

                let mut release = None;
                if release_group.is_some() {
                    release = Self::db_merge_model(&db, &media_file.release().model(), &release_group);
                    Self::lazy_link(&db, &release, &release_group);
                    Self::lazy_link(&db, &release, &artist); // TODO should be album artist probably
                }

                let mut medium = None;
                if release.is_some() {
                    medium = Self::db_merge_model(&db, &media_file.medium().model(), &release);
                    Self::lazy_link(&db, &medium, &release);
                    Self::lazy_link(&db, &medium, &artist);
                }

                // TODO can improve further by say allowing creation when there is a mbid
                // even if there is no release/medium
                let mut track = None;
                if medium.is_some() {
                    track = Self::db_merge_model(&db, &media_file.track().model(), &medium);
                    Self::lazy_link(&db, &track, &medium);
                    Self::lazy_link(&db, &track, &artist);
                }

                for genre in media_file.genres() {
                    let genre = Self::db_merge_model(&db, &genre.model(), &None);
                    Self::lazy_link(&db, &genre, &artist);
                    Self::lazy_link(&db, &genre, &release_group);
                    Self::lazy_link(&db, &genre, &release);
                    Self::lazy_link(&db, &genre, &medium);
                    Self::lazy_link(&db, &genre, &track);
                }

                for visual in media_file.visuals.iter() {
                    let image = image::load_from_memory(&visual.data);
                    if image.is_err() {
                        continue;
                    }
                    let image = image.unwrap();
                    // TODO this is temporary, just checking if no image has been
                    // set on the release at all, and if so, linking it up to everything
                    // we've created.
                    if release.is_some() && db.list(&Picture::default().model(), release.as_ref()).unwrap().count() == 0 {
                        let mut picture = Picture::default();
                        picture.set_image(&image);
                        let picture = db.insert(&picture.model()).unwrap();
                        Self::lazy_link(&db, &Some(picture.clone()), &artist);
                        Self::lazy_link(&db, &Some(picture.clone()), &release_group);
                        Self::lazy_link(&db, &Some(picture.clone()), &release);
                        Self::lazy_link(&db, &Some(picture.clone()), &track);
                    }
                }



                count += 1;
            }
        }
        log::info!("Scanned {}, skipped {} in {}ms", 
            count, 
            skipped, 
            now.elapsed().as_millis());
    }

    /// Links the two Models if they are both Some. Reduces boilerplate.
    fn lazy_link(db: &dyn Db, l: &Option<Model>, r: &Option<Model>) {
        if l.is_some() && r.is_some() {
            db.link(&l.clone().unwrap(), &r.clone().unwrap()).unwrap()
        }
    }

    fn db_merge_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
        // find a matching model to the specified, merge, save
        let matching = Self::find_matching_model(db, model, parent);
        if let Some(matching) = matching {
            let merged = Self::merge_model(&model, &matching);
            return Some(db.insert(&merged).unwrap())
        }
        // if not, insert the new one and link it to the parent
        else {
            if Self::model_valid(model) {
                return Some(db.insert(model).unwrap())
            }
        }
        None
    }

    fn find_matching_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
        // This needs to use the scoring and sorting, but yea.
        db.list(&model, parent.as_ref()).unwrap()
            .filter(|model_opt| Self::compare_models(&model, model_opt))
            .next()
    }

    fn compare_models(l: &Model, r: &Model) -> bool {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => {
                // TODO needs to include disambiguation - I think we're getting back to mergability.
                (l.name.is_some() && l.name == r.name)
                || (l.known_ids.musicbrainz_id.is_some() && l.known_ids.musicbrainz_id == r.known_ids.musicbrainz_id)
            },
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => {
                l.title.is_some() && l.title == r.title
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
            _ => todo!()
        }
    }

    fn merge_model(l: &Model, r: &Model) -> Model {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => Artist::merge(l.clone(), r.clone()).model(),
            (Model::Genre(l), Model::Genre(r)) => Genre::merge(l.clone(), r.clone()).model(),
            (Model::Medium(l), Model::Medium(r)) => Medium::merge(l.clone(), r.clone()).model(),
            (Model::Release(l), Model::Release(r)) => Release::merge(l.clone(), r.clone()).model(),
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => ReleaseGroup::merge(l.clone(), r.clone()).model(),
            (Model::Track(l), Model::Track(r)) => Track::merge(l.clone(), r.clone()).model(),
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
}

impl Plugin for MediaFilesPlugin {
    fn init(&self, librarian: &Librarian) {
        *self.librarian.lock().unwrap() = Some(librarian.clone());
        self.rescan();
    }

    fn set_network_mode(&self, _network_mode: &NetworkMode) {
        // Don't care, local only.
    }
}

