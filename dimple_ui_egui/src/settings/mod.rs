use config::Config;
use serde::{Deserialize, Serialize};

use crate::music_library::LibraryConfig;

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
