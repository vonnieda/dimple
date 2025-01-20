use image::DynamicImage;

use crate::{library::Library, model::{Artist, Dimage, Genre, Model, Release, Track}};

use super::plugin_host::PluginHost;

pub trait Plugin: Send + Sync {
    fn display_name(&self) -> String { 
        self.type_name() 
    }
    
    fn type_name(&self) -> String;
    
    fn configuration(&self) -> String { 
        "".to_string() 
    }
    
    fn set_configuration(&mut self, _config: &str) { 

    }

    fn status(&self) -> Option<String> { 
        None 
    }

    fn progress(&self) -> Option<f32> { 
        None 
    }

    fn metadata(&self, _host: &PluginHost, _library: &Library, _model: &dyn Model) 
        -> Result<Option<Box<dyn Model>>, anyhow::Error> {
        Ok(None)
    }

    // fn image(&self, _host: &PluginHost, _library: &Library, _track: &Track, _model: &PluginModel) -> Result<Option<DynamicImage>, anyhow::Error> {
    //     Ok(None)
    // }
}
