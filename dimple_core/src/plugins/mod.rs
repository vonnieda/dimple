pub mod plugin_host;
pub mod lrclib;
pub mod musicbrainz;
pub mod wikidata;
pub mod s3_api_sync;

use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::{library::Library, model::Track};

pub const USER_AGENT: &str = "Dimple/0.0.1 +https://github.com/vonnieda/dimple +jason@vonnieda.org";

pub trait Plugin: Send + Sync {
    fn display_name(&self) -> String;
    fn type_name(&self) -> String;
    fn configuration(&self) -> String;
    fn set_configuration(&mut self, config: &str);
    fn status(&self) -> String;

    fn get_track_lyrics(&self, _library: &Library, _track: &Track) 
            -> Option<String> {
        None
    }

    fn get_track_coverart(&self, _library: &Library, _track: &Track) 
            -> Option<DynamicImage> {
        None
    }

    fn sync(&self, _library: &Library) -> Result<(), ()> {
        Ok(())
    }
}

