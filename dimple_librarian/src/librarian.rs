use std::sync::RwLock;

use colored::Colorize;
use dimple_core::{library::{Library, LibraryEntity}, model::DimpleArtist};
use dimple_sled_library::sled_library::SledLibrary;
use image::DynamicImage;
use rayon::prelude::*;

// TODO need a favicon service


pub struct Librarian {
    /// TODO It feels like it's about time to retire SledLibrary, and move the
    /// Sled stuff here. I'm going to need additional trees for configs,
    /// user images, and other stuff. I had considered just having SledLibrary
    /// take a Tree, and then I could control the root here, but then SledLibrary
    /// is basically doing nothing but serializing which is easy enough to do here.
    /// Then I don't have to be retricted to the Library operations for the stuff
    /// that happens here.
    local_library: SledLibrary,
    libraries: RwLock<Vec<Box<dyn Library>>>,
}

impl Default for Librarian {
    fn default() -> Self {
        Self { 
            local_library: SledLibrary::new("local_library", "local_library"),
            libraries: Default::default(), 
        }
    }    
}

impl Librarian {
    pub fn add_library(&self, library: Box<dyn Library>) {
        self.libraries.write().unwrap().push(library);
    }

    /// Generate some kind of cool artwork for the entity to be used as a
    /// default. Being part of Librarian, it can use data from the library
    /// to create the image.
    pub fn generate_masterpiece(&self, entity: &LibraryEntity, width: u32, 
        height: u32) -> DynamicImage {


        // https://stackoverflow.com/questions/76741218/in-slint-how-do-i-render-a-self-drawn-image


        DynamicImage::new_rgb8(width, height)
    }

    pub fn thumbnail(&self, entity: &LibraryEntity, width: u32, height: u32) -> Option<DynamicImage> {
        self.local_library.images.get(&entity.id(), width, height)
            .or_else(|| {
                self.image(entity).map(|dyn_image| {
                    self.local_library.set_image(entity, &dyn_image);
                    dyn_image
                })
            })
            .or_else(|| {
                // TODO magic, used 1000x1000 cause that's what fanart.tv uses
                // for artist thumbs. Could probably go higher res for an
                // "original" quality image.
                self.local_library.set_image(entity, 
                    &self.generate_masterpiece(entity, 1000, 1000));
                self.local_library.images.get(&entity.id(), width, height)
            })
    }
}

impl Library for Librarian {
    fn name(&self) -> String {
        "Librarian".to_string()
    }

    fn search(&self, query: &str) -> Box<dyn Iterator<Item = dimple_core::library::LibraryEntity>> {
        // TODO include local
        // TODO remove dupes
        log::debug!("{}: {}", "Search".cyan(), query.blue());
        let merged: Vec<LibraryEntity> = self.libraries.read().unwrap().iter()
            .flat_map(|lib| {
                log::debug!("  {} {}", "✔".bright_green(), lib.name().green());
                lib.search(query)
            })
            .collect();
        Box::new(merged.into_iter())
    }    

    fn artists(&self) -> Box<dyn Iterator<Item = DimpleArtist>> {
        self.local_library.artists()
    }


    /// Librarian extends the contract of fetch by combining the results of
    /// fetching from each registered library into one result. This result is
    /// cached in the local library and updated whenever fetch_force is called.
    /// 
    /// The local library is queried first, and if a value is found it is used
    /// as the input for the other queries. The ensures that libraries that
    /// depend on content from Musicbrainz for their query can access it, if we
    /// have it.
    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        // TODO include timing
        let fetch_and_log = |lib: &dyn Library, entity: &LibraryEntity| {
            lib.fetch(entity)
                .map(|ent| { 
                    log::debug!("  {} {}", "✔".bright_green(), lib.name().bright_green());
                    ent
                })
                .or_else(|| {
                    log::debug!("  {} {}", "✗".bright_red(), lib.name().bright_red());
                    None
                })
        };

        let store_and_log = |entity: LibraryEntity| -> LibraryEntity {
            match &entity {
                LibraryEntity::Artist(artist) => self.local_library.set_artist(artist),
                LibraryEntity::ReleaseGroup(r) => self.local_library.set_release_group(r),
                LibraryEntity::Release(r) => self.local_library.set_release(r),
                LibraryEntity::Genre(_) => todo!(),
                LibraryEntity::Track(_) => todo!(),
            };
            entity
        };

        fn longer(a: String, b: String) -> String {
            if a.len() > b.len() {
                a
            }
            else {
                b
            }
        }

        fn merge_vec<T>(a: Option<Vec<T>>, b: Option<Vec<T>>) -> Option<Vec<T>> {
            if a.is_none() && b.is_none() {
                return None;
            }
            if a.is_some() & b.is_some() {
                let a = a.unwrap();
                let b = b.unwrap();
                if a.len() >= b.len() {
                    Some(a)
                } 
                else {
                    Some(b)
                }
            }
            else if a.is_some() {
                return a;
            }
            else {
                return b;
            }
        }

        // TODO this sucks, it needs to be genericd so the two variants have
        // to be of the same type.
        // TODO merge de-dup etc.
        let merge = |a: Option<LibraryEntity>, b: LibraryEntity| -> Option<LibraryEntity> {
            let base = a.unwrap_or(b.clone());
            match base {
                LibraryEntity::Artist(mut base) => {
                    if let LibraryEntity::Artist(b) = b {
                        base.disambiguation = longer(base.disambiguation, b.disambiguation);
                        base.genres = merge_vec(base.genres, b.genres);
                        base.id = longer(base.id, b.id);
                        base.name = longer(base.name, b.name);
                        base.relations = merge_vec(base.relations, b.relations);
                        base.release_groups = merge_vec(base.release_groups, b.release_groups);
                        base.summary = base.summary.or(b.summary.clone());
                    }
                    Some(LibraryEntity::Artist(base))
                },
                LibraryEntity::ReleaseGroup(mut base) => {
                    if let LibraryEntity::ReleaseGroup(b) = b {
                        base.disambiguation = longer(base.disambiguation, b.disambiguation);
                        base.first_release_date = longer(base.first_release_date, b.first_release_date);
                        base.genres = merge_vec(base.genres, b.genres);
                        base.id = longer(base.id, b.id);
                        base.primary_type = longer(base.primary_type, b.primary_type);
                        base.relations = merge_vec(base.relations, b.relations);
                        base.releases = merge_vec(base.releases, b.releases);
                        base.summary = base.summary.or(b.summary.clone());
                        base.title = longer(base.title, b.title);
                        base.artists = merge_vec(base.artists, b.artists);
                    }
                    Some(LibraryEntity::ReleaseGroup(base))
                },
                LibraryEntity::Release(mut base) => {
                    if let LibraryEntity::Release(b) = b {
                        base.artists = merge_vec(base.artists, b.artists);
                        base.date = longer(base.date, b.date);
                        base.barcode = longer(base.barcode, b.barcode);
                        base.status = longer(base.status, b.status);

                        base.disambiguation = longer(base.disambiguation, b.disambiguation);
                        // base.first_release_date = longer(base.first_release_date, b.first_release_date);
                        base.genres = merge_vec(base.genres, b.genres);
                        base.id = longer(base.id, b.id);
                        // base.primary_type = longer(base.primary_type, b.primary_type);
                        base.relations = merge_vec(base.relations, b.relations);
                        // base.releases = merge_vec(base.releases, b.releases);
                        base.summary = base.summary.or(b.summary.clone());
                        base.title = longer(base.title, b.title);
                    }
                    Some(LibraryEntity::Release(base))
                },
                _ => todo!(),
            }
        };

        log::debug!("{} {} ({})", "Fetch".green(), entity.name().blue(), entity.id().yellow());        

        let local_result: Option<LibraryEntity> = fetch_and_log(&self.local_library, entity);

        if local_result.clone().is_some_and(|x| x.fetched()) {
            return local_result;
        }

        let query_result: &LibraryEntity = local_result.as_ref().unwrap_or(entity);
        
        let first_results: Vec<LibraryEntity> = self.libraries.read().ok()?.par_iter()
            .filter_map(|lib| fetch_and_log(lib.as_ref(), query_result))
            .collect();
        
        let first_result = first_results.into_iter()
            .fold(local_result, merge)?;

        // All good up to here. Now just need to not re-query the ones that 
        // returned a valid ret
        let remote_results2: Vec<LibraryEntity> = self.libraries.read().ok()?.par_iter()
            .filter_map(|lib| fetch_and_log(lib.as_ref(), &first_result))
            .collect();

        // TODO this is all working great, but I can't double query everyone.
        // So I think I need to exclude the ones return Some.
        
        remote_results2.into_iter()
            .fold(None, merge)
            .map(|f| {
                match f {
                    LibraryEntity::Artist(a) => {
                        let mut a = a.clone();
                        a.fetched = true;
                        LibraryEntity::Artist(a)
                    },
                    LibraryEntity::ReleaseGroup(r) => {
                        let mut r = r.clone();
                        r.fetched = true;
                        LibraryEntity::ReleaseGroup(r)
                    },
                    LibraryEntity::Release(r) => {
                        let mut r = r.clone();
                        r.fetched = true;
                        LibraryEntity::Release(r)
                    },
                    _ => todo!(),
                }
            })
            .map(store_and_log)
            .inspect(|f| {
                log::debug!("{:?}", &f);
            })
    }

    /// If there is an image stored in the local library for the entity return
    /// it, otherwise search the attached libraries for one. If one is found,
    /// cache it in the local library and return it. Furture requests will be
    /// served from the cache.
    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        let fetch_and_log = |lib: &dyn Library, entity: &LibraryEntity| {
            let result = lib.image(entity);
            if result.is_some() {
                log::debug!("  {} {}", "✔".bright_green(), lib.name().green());
            }
            else {
                log::debug!("  {} {}", "✗".bright_red(), lib.name().bright_red());
            }
            result
        };

        log::debug!("{} {} ({})", "Image".magenta(), entity.name().blue(), entity.id().yellow());
        fetch_and_log(&self.local_library, entity)
            .or_else(|| {
                self.libraries.read().ok()?.par_iter()
                    .find_map_first(|lib| fetch_and_log(lib.as_ref(), entity))
                    .map(|dyn_image| {
                        self.local_library.set_image(entity, &dyn_image);
                        dyn_image
                    })
            }
        )
    }
}
