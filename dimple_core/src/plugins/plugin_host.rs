use std::sync::{Arc, RwLock};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{library::Library, merge::CrdtRules, model::Track};

use super::plugin::Plugin;

#[derive(Default, Clone)]
pub struct PluginHost {
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
}

/// Right away, I need:
/// - lrclib lyrics (already have, fit API)
/// - musicbrainz links (needed for summary)
/// - wikidata summary
/// - tadb artist artwork
/// - fanart artist artwork
/// - caa release artwork
impl PluginHost {
    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    }

    pub fn metadata(&self, library: &Library, track: &Track) -> Result<Option<Track>, anyhow::Error> {
        let results = self.plugins
            .read()
            .unwrap()
            .par_iter()
            .filter_map(|plugin| plugin.metadata(library, track).ok())
            .reduce(|| None, CrdtRules::merge);
        Ok(results)
    }

    pub fn image(&self, _library: &Library, _track: &Track) -> Option<Track> {
        todo!()
        // Some(self.plugins
        //     .read()
        //     .unwrap()
        //     .par_iter()
        //     .filter_map(|plugin| plugin.metadata(library, track))
        //     .reduce(Track::default, Track::merge))
    }
}

#[cfg(test)]
mod tests { 
    use crate::{
        library::Library,
        model::{Artist, ArtistRef, Track}, plugins::{example::ExamplePlugin, lrclib::LrclibPlugin},
    };

    use super::PluginHost;

    #[test]
    fn it_works() {
        let plugins = PluginHost::default();
        plugins.add_plugin(Box::new(ExamplePlugin::default()));
        plugins.add_plugin(Box::new(LrclibPlugin::default()));

        let library = Library::open_memory();
        let artist = library.save(&Artist {
            name: Some("Metallica".to_string()),
            ..Default::default()
        });
        let track = library.save(&Track {
            title: Some("Master of Puppets".to_string()),
            ..Default::default()
        });
        let _ac = library.save(&ArtistRef {
            model_key: track.key.clone().unwrap(),
            artist_key: artist.key.clone().unwrap(),
            ..Default::default()
        });

        // println!("{:?}", plugins.lyrics(&library, &track));
        println!("{:?}", plugins.metadata(&library, &track));
    }
}



