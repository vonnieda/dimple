use std::sync::Arc;

use config::Config;
use dimple_core::library::LibraryHandle;
use dimple_navidrome_library::navidrome_library::{NavidromeLibraryConfig, NavidromeLibrary};
use dimple_sled_library::sled_library::{SledLibraryConfig, SledLibrary};
use serde::{Deserialize, Serialize};

use crate::librarian::Librarian;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LibraryConfig {
    Navidrome(NavidromeLibraryConfig),
    Sled(SledLibraryConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub libraries: Vec<LibraryConfig>,
}

impl From<Config> for Settings {
    fn from(config: Config) -> Self {
        config.try_deserialize().unwrap()
    }
}

impl Default for Settings {
    fn default() -> Self {
        config::Config::builder()
            .add_source(config::File::with_name("config.yaml"))
            .build()
            .unwrap()
            .into()
    }
}

impl From<Vec<LibraryConfig>> for Librarian {
    fn from(configs: Vec<LibraryConfig>) -> Self {
        let mut librarian = Self::default();
        for config in configs {
            let library: LibraryHandle = match config {
                LibraryConfig::Navidrome(config) => Arc::new(NavidromeLibrary::from(config)),
                LibraryConfig::Sled(config) => Arc::new(SledLibrary::from(config)),
            };
            librarian.add_library(library);
        }
        librarian
    }
}

impl From<Settings> for Librarian {
    fn from(settings: Settings) -> Self {
        Librarian::from(settings.libraries)
    }
}
