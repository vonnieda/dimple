// Example of how to load plugins dynamically, thanks to Claude
// https://claude.ai/chat/e7fd83d8-6c8b-443d-afdb-471b4fe0fcbe

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    // Add more fields as needed
}

#[async_trait]
pub trait MetadataPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn fetch_metadata(&self, query: &str) -> Result<Vec<Metadata>, Box<dyn std::error::Error>>;
}

use std::sync::Arc;
use libloading::{Library, Symbol};

type PluginCreate = unsafe fn() -> *mut dyn MetadataPlugin;

pub struct PluginLoader {
    plugins: Vec<Arc<dyn MetadataPlugin>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        PluginLoader { plugins: Vec::new() }
    }

    pub fn load_plugin(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let lib = Library::new(path)?;
            let func: Symbol<PluginCreate> = lib.get(b"_plugin_create")?;
            let plugin = Arc::from_raw(func());
            self.plugins.push(plugin);
            // Intentionally forget the library to keep it loaded
            std::mem::forget(lib);
        }
        Ok(())
    }

    pub fn get_plugins(&self) -> &[Arc<dyn MetadataPlugin>] {
        &self.plugins
    }
}

pub struct PluginManager {
    loader: PluginLoader,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            loader: PluginLoader::new(),
        }
    }

    pub fn load_plugins(&mut self, plugin_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "so" || ext == "dll") {
                self.loader.load_plugin(path.to_str().unwrap())?;
            }
        }
        Ok(())
    }

    pub fn get_plugins(&self) -> &[Arc<dyn MetadataPlugin>] {
        self.loader.get_plugins()
    }
}

struct ExamplePlugin;

#[async_trait]
impl MetadataPlugin for ExamplePlugin {
    fn name(&self) -> &str {
        "Example Plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn fetch_metadata(&self, query: &str) -> Result<Vec<Metadata>, Box<dyn std::error::Error>> {
        // Implement metadata fetching logic here
        Ok(vec![Metadata {
            title: Some(format!("Example Song for {}", query)),
            artist: Some("Example Artist".to_string()),
            album: None,
            year: None,
            genre: None,
        }])
    }
}

#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut dyn MetadataPlugin {
    Box::into_raw(Box::new(ExamplePlugin))
}

async fn fetch_metadata_from_plugins(query: &str) -> Result<Vec<Metadata>, Box<dyn std::error::Error>> {
    let mut manager = PluginManager::new();
    manager.load_plugins("path/to/plugins/directory")?;

    let mut all_metadata = Vec::new();
    for plugin in manager.get_plugins() {
        let metadata = plugin.fetch_metadata(query).await?;
        all_metadata.extend(metadata);
    }

    Ok(all_metadata)
}