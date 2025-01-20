use serde::{Deserialize, Serialize};

use crate::{library::Library, model::{Model, Track}};

use super::{plugin::Plugin, plugin_host::PluginHost};

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

    fn metadata(&self, _host: &PluginHost, _library: &Library, model: &dyn Model) 
        -> Result<Option<Box<dyn Model>>, anyhow::Error> {

        if let Some(track) = model.as_any().downcast_ref::<Track>() {
            return Ok(Some(Box::new(Track {
                lyrics: Some(format!("(unrecognizable shrieking)")),
                ..Default::default()
            })))
        }

        Ok(None)
    }    
}
