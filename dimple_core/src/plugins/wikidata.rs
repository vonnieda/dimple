use serde::{Deserialize, Serialize};

use super::Plugin;

#[derive(Default)]
pub struct WikiDataPlugin {
    config: WikiDataPluginConfig,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct WikiDataPluginConfig {    
    pub url: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl Plugin for WikiDataPlugin {
    fn type_name(&self) -> String {
        "WikiDataPlugin".to_string()
    }

    fn display_name(&self) -> String {
        "WikiData and WikiPedia".to_string()
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
