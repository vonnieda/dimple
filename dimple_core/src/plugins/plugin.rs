use std::path::Path;

use anyhow::Result;

use crate::{librarian::{ArtistMetadata, ReleaseGroupMetadata, ReleaseMetadata, SearchResults, TrackMetadata}, library::Library, model::{Artist, Dimage, DimpleEntity, Release, ReleaseGroup, Track, TrackSource}};

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

    fn artist_metadata(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _artist: &Artist) 
        -> Result<Option<ArtistMetadata>> {
        
        Ok(None)
    }

    fn artist_release_groups(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _artist: &Artist) 
        -> Result<Vec<ReleaseGroupMetadata>> {
        
        Ok(vec![])
    }

    fn artist_releases(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _artist: &Artist) 
        -> Result<Vec<ReleaseMetadata>> {
        
        Ok(vec![])
    }

    fn track_metadata(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _track: &Track) 
        -> Result<Option<TrackMetadata>> {
        
        Ok(None)
    }

    fn release_group_metadata(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _release_group: &ReleaseGroup) 
        -> Result<Option<ReleaseGroupMetadata>> {
        
        Ok(None)
    }

    fn release_group_releases(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _release_group: &ReleaseGroup) 
        -> Result<Vec<ReleaseMetadata>> {
        
        Ok(vec![])
    }

    fn release_metadata(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _release: &Release) 
        -> Result<Option<ReleaseMetadata>> {
        
        Ok(None)
    }

    fn search(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _query: &str) 
        -> Result<SearchResults> {
        
        Ok(SearchResults::default())
    }

    // TODO add DimageKind filter
    fn image(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _for_entity: &DimpleEntity) 
        -> Result<Option<Dimage>> {
        
        Ok(None)
    }

    fn track_sources(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _track: &Track) 
        -> Result<Vec<TrackSource>> {
        
        Ok(vec![])
    }

    // TODO I think things have settled enough now that we can move the importers
    // into plugins. They should return an entity if possible, or none if returning
    // an entity doesn't make sense, and only error to indicate a true failure.
    fn import(&self, 
        _plugins: &Plugins, 
        _library: &Library, 
        _path: &Path) 
        -> Result<Option<DimpleEntity>> {
        Ok(None)
    }
}
