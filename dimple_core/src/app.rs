use std::{path::PathBuf, sync::Arc};

use crate::{cache::Cache, config::Config, library::Library, player::Player, plugins::plugins::Plugins};

/// Pulls together functionality that most Dimple player app implementations
/// will use. This is the high level API for Dimple.
/// 
/// I guess this should probably be it's own crate. We'd have dimple_core,
/// dimple_app, dimple_app_slint

pub struct DimpleApp {
    config: Arc<Config>,
    library: Arc<Library>,
    cache: Arc<Cache>,
    player: Arc<Player>,
    plugins: Arc<Plugins>,
}

/// TODO
/// Collection:
///     - recommendations()
///     - playlists()
///     - artists()
///     - releases()
///     - songs()
///     - genres()
///     - history()
///     - assets()
///     - set_liked()
///     - set_lyrics()
/// Assets:
/// Player:
///     - queue()
///     - play()
///     - pause()
///     - seek()
/// Search:
///     - query()
/// Import:
/// Settings:
/// Plugins:
///     - plugins()
///     - set_enabled()
/// 
/// I want to collect a list of the things the slint UI actually needs:
/// 
/// Query subscriptions → reactive data streams
/// Complex JOINs → simple service methods
/// Transaction handling → business logic layer
/// Background tasks (metadata refresh)
impl DimpleApp {
    pub fn open_memory() -> anyhow::Result<Self> {
        todo!()
    }

    /// Opens the user default path using ProjectDirs.
    pub fn open_default() -> anyhow::Result<Self> {
        // let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        // let mut data_dir = dirs.data_dir().to_path_buf();
        // let mut config_dir = dirs.config_dir().to_path_buf();
        // let mut cache_dir = dirs.cache_dir().to_path_buf();
        // if let Ok(root) = env::var("DIMPLE_ROOT") {
        //     let root_dir = Path::new(&root.to_string()).to_path_buf();
        //     data_dir = root_dir.join("data").to_path_buf();
        //     config_dir = root_dir.join("config").to_path_buf();
        //     cache_dir = root_dir.join("cache").to_path_buf();
        // }
        // let library_path = data_dir.join("library.db");
        // let config_path = config_dir.join("config.db");
        // let image_cache_dir = cache_dir.join("image_cache");
        // dbg!(&data_dir, &cache_dir, &library_path, &image_cache_dir, &config_path);
        // std::fs::create_dir_all(&data_dir).unwrap();
        // std::fs::create_dir_all(&config_dir).unwrap();
        // std::fs::create_dir_all(&cache_dir).unwrap();
        // std::fs::create_dir_all(&image_cache_dir).unwrap();

        // let library = Library::open(library_path.to_str().unwrap());
        // let config = Config::new(Db::open(config_path.to_str().unwrap()).unwrap()).unwrap();
        // let player = Player::new(Arc::new(library.clone()));
        // let tasks = Tasks::new();
        // let plugins = Plugins::new(cache_dir.to_str().unwrap());
        // plugins.add_default_plugins();
        // let _ = plugins.initialize(&library, &tasks);
        // let librarian = Librarian::new(&library, &plugins);
        todo!()
    }

    pub fn open_path(path: &PathBuf) -> anyhow::Result<Self> {
        todo!()
    }

    pub fn artists(&self) -> ArtistsCollection {
        todo!()
    }
}

pub struct ArtistsCollection {

}

impl ArtistsCollection {
    pub fn list(&self) -> anyhow::Result<CollectionQuery<Vec<ArtistView>>> {
        todo!()
    }

    pub fn get(&self, id: &str) -> anyhow::Result<CollectionQuery<ArtistView>> {
        todo!()
    }
}

pub struct CollectionQuery<R> {
    r: R,
}

impl <R> CollectionQuery<R> {
    fn observe(&self, callback: impl Fn(R) -> ()) {
        todo!()
    }

    fn once(&self) {
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ArtistView {
    pub name: String,
    pub genres: Vec<GenreView>,
}

#[derive(Clone, Debug, Default)]
pub struct GenreView {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use crate::app::{ArtistView, DimpleApp};

    /// Get ArtistView by key
    /// Verify it includes genres
    #[test]
    fn artist_details() -> anyhow::Result<()> {
        let app = DimpleApp::open_memory()?;
        app.artists().get("123-123-123-123")?.observe(|artist: ArtistView| {
            println!("artist updated {:?}", artist);
        });
        Ok(())
    }
}
