pub mod plugin_host;
pub mod lrclib;
pub mod musicbrainz;
pub mod wikidata;
pub mod s3_api_sync;
pub mod example;

use crate::{library::Library, model::{Artist, Release, Track}};

pub const USER_AGENT: &str = "Dimple/0.0.1 +https://github.com/vonnieda/dimple +jason@vonnieda.org";

pub trait Plugin: Send + Sync {
    fn display_name(&self) -> String { self.type_name() }
    fn type_name(&self) -> String;
    fn configuration(&self) -> String { "".to_string() }
    fn set_configuration(&mut self, config: &str) { }
    fn status(&self) -> String;

    // TODO These Options should be Results but I steadfastly refuse to learn
    // all the Error boilerplate.

    fn lyrics(&self, _library: &Library, _track: &Track) -> Option<String> {
        None
    }

    fn metadata(&self, _library: &Library, _track: &Track) 
            -> Option<(Option<Artist>, Option<Release>, Track)> {
        None
    }
}

