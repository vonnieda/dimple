use config::Config;
use dimple_navidrome_library::navidrome_library::NavidromeLibraryConfig;
use dimple_sled_library::sled_library::SledLibraryConfig;
use serde::{Deserialize, Serialize};

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
