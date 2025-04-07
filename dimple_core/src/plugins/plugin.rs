use image::DynamicImage;

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Release, Track}};

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

    // fn metadata(&self, _host: &PluginHost, _library: &Library, _model: &dyn Model) 
    //     -> Result<Option<MetadataResponse>, anyhow::Error> {
    //     Ok(None)
    // }

    fn artist_metadata(&self, _host: &PluginHost, _library: &Library, _artist: &Artist) -> Result<Option<ArtistMetadata>, anyhow::Error> {
        Ok(None)
    }

    fn track_metadata(&self, _host: &PluginHost, _library: &Library, _track: &Track) -> Result<Option<TrackMetadata>, anyhow::Error> {
        Ok(None)
    }

    fn release_metadata(&self, _host: &PluginHost, _library: &Library, _release: &Release) -> Result<Option<ReleaseMetadata>, anyhow::Error> {
        Ok(None)
    }

    // fn image(&self, _host: &PluginHost, _library: &Library, _track: &Track, _model: &PluginModel) -> Result<Option<DynamicImage>, anyhow::Error> {
    //     Ok(None)
    // }    

    fn search(&self, _host: &PluginHost, _library: &Library, _query: &str) -> Result<SearchResults, anyhow::Error> {
        Ok(SearchResults::default())
    }

    fn artist_image(&self, _host: &PluginHost, _library: &Library, _artist: &Artist) -> Result<Option<DynamicImage>, anyhow::Error> {
        Ok(None)
    }
}
