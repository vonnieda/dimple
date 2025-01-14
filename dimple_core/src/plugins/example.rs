use serde::{Deserialize, Serialize};

use super::plugin::Plugin;

#[derive(Default)]
pub struct ExamplePlugin {
    config: ExamplePluginConfig,
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct ExamplePluginConfig {    
    pub url: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl Plugin for ExamplePlugin {
    fn display_name(&self) -> String {
        "Example".to_string()
    }

    fn type_name(&self) -> String {
        "ExamplePlugin".to_string()
    }

    fn set_configuration(&mut self, config: &str) {
        self.config = serde_json::from_str(config).unwrap();
    }

    fn configuration(&self) -> String {
        serde_json::to_string(&self.config).unwrap()
    }
    
    fn metadata(&self, _library: &crate::library::Library, track: &crate::model::Track) -> Result<Option<crate::model::Track>, anyhow::Error> {
        let mut track = track.clone();
        track.lyrics = Some(format!("(unrecognizable shrieking)"));
        Ok(Some(track))
    }
}
