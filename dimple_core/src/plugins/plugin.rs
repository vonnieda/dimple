use anyhow::Result;

use crate::{librarian::{ArtistMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Dimage, DimpleEntity, Release, Track, TrackSource}};

use super::plugins::Plugins;

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

    fn artist_metadata(&self, _host: &Plugins, _library: &Library, _artist: &Artist) -> Result<Option<ArtistMetadata>> {
        Ok(None)
    }

    fn track_metadata(&self, _host: &Plugins, _library: &Library, _track: &Track) -> Result<Option<TrackMetadata>> {
        Ok(None)
    }

    fn release_metadata(&self, _host: &Plugins, _library: &Library, _release: &Release) -> Result<Option<ReleaseMetadata>> {
        Ok(None)
    }

    fn search(&self, _host: &Plugins, _library: &Library, _query: &str) -> Result<SearchResults> {
        Ok(SearchResults::default())
    }

    // TODO add DimageKind filter
    fn image(&self, _host: &Plugins, _library: &Library, _model: &DimpleEntity) -> Result<Option<Dimage>> {
        Ok(None)
    }

    fn track_sources(&self, _plugins: &Plugins, _library: &Library, _track: &Track) -> Result<Vec<TrackSource>> {
        Ok(vec![])
    }
}
