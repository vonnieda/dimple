use std::sync::{Arc, RwLock};

use super::Plugin;

#[derive(Default, Clone)]
pub struct PluginHost {
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
}

impl PluginHost {
    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    } 
}

