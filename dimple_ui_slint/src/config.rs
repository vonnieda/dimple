use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct Config {
    path: String,
    config_file: ConfigFile,
}

impl Config {
    // pub fn open(path: &str) -> Config {
    //     Config {
    //         path: path.to_string(),
    //         config_file: ConfigFile::default(),
    //     }
    // }

    pub fn on_change(&self) {
        todo!()
    }

    pub fn emit_change(&self, event: &str) {

    }

    pub fn offline_mode(&self) -> bool {
        self.config_file.offline_mode
    }

    pub fn set_offline_mode(&mut self, value: bool) {
        self.config_file.offline_mode = value;
        self.emit_change("offline_mode");
    }

    pub fn plugin_config(&self) -> Vec<PluginConfig> {
        self.config_file.plugin_config.clone()
    }

    pub fn add_plugin_config(&mut self, config: &PluginConfig) {
        self.config_file.plugin_config.push(config.clone());
    }

    fn load(&self) {

    }

    fn save(&self) {

    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub offline_mode: bool,
    pub plugin_config: Vec<PluginConfig>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub key: String,
    pub type_name: String,
    pub enabled: bool,
    pub config: String,
}
