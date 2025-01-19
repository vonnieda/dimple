use image::DynamicImage;

use crate::{library::Library, model::{Dimage, Track}};

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

    fn metadata(&self, _library: &Library, _track: &Track) -> Result<Option<Track>, anyhow::Error> {
        Ok(None)
    }

    fn image(&self, _library: &Library, _track: &Track, _kind: Dimage) -> Result<Option<DynamicImage>, anyhow::Error> {
        Ok(None)
    }

    fn artist_metadata(&self, _library: &Library, _artist: &Track) -> Result<Option<Track>, anyhow::Error> {
        Ok(None)
    }
}

