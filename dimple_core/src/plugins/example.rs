pub struct ExamplePlugin {
    config: ExamplePluginConfig,
}

#[derive(Serialize, Deserialize, Clone)]
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

    fn status(&self) -> String {
        "Ready".to_string()
    }

    fn get_track_lyrics(&self, _library: &Library, _track: &Track) 
            -> Option<String> {
        Some(format!("(unrecognizable shrieking)"))
    }
}
