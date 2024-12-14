use serde::{Deserialize, Serialize};

use super::Plugin;

#[derive(Default)]
pub struct MusicBrainzPlugin {
    config: MusicBrainzPluginConfig,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct MusicBrainzPluginConfig {    
    pub url: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl Plugin for MusicBrainzPlugin {
    fn type_name(&self) -> String {
        "MusicBrainzPlugin".to_string()
    }

    fn display_name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn set_configuration(&mut self, config: &str) {
        self.config = serde_json::from_str(config).unwrap();
    }

    fn configuration(&self) -> String {
        serde_json::to_string(&self.config).unwrap()
    }

    fn status(&self) -> String {
        "Ready".to_string()
    }
}
