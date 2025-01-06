use std::sync::{Arc, RwLock};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{library::Library, model::{Artist, Release, Track}};

use super::Plugin;

#[derive(Default, Clone)]
pub struct PluginHost {
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
}

impl PluginHost {
    pub fn add_plugin(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().unwrap().push(plugin);
    }

    pub fn lyrics(&self, library: &Library, track: &Track) -> Vec<String> {
        self.plugins
            .read()
            .unwrap()
            .par_iter()
            .filter_map(|plugin| plugin.lyrics(library, track))
            .collect()
    }

    pub fn metadata(&self, library: &Library, track: &Track) -> Vec<(Option<Artist>, Option<Release>, Track)> {
        self.plugins
            .read()
            .unwrap()
            .par_iter()
            .filter_map(|plugin| plugin.metadata(library, track))
            .collect()    
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        library::Library,
        model::{Artist, ArtistRef, Track},
        plugins::{
            example::ExamplePlugin, lrclib::LrclibPlugin, musicbrainz::MusicBrainzPlugin,
            s3_api_sync::S3ApiSyncPlugin, wikidata::WikidataPlugin,
        },
    };

    use super::PluginHost;

    #[test]
    fn it_works() {
        let plugins = PluginHost::default();
        plugins.add_plugin(Box::new(ExamplePlugin::default()));
        plugins.add_plugin(Box::new(LrclibPlugin::default()));
        plugins.add_plugin(Box::new(MusicBrainzPlugin::default()));
        plugins.add_plugin(Box::new(S3ApiSyncPlugin::default()));
        plugins.add_plugin(Box::new(WikidataPlugin::default()));

        let library =
            Library::open("file:2fe945e2-8191-43fd-80a8-5a99efea641d?mode=memory&cache=shared");
        let artist = library.save(&Artist {
            name: Some("Metallica".to_string()),
            ..Default::default()
        });
        let track = library.save(&Track {
            title: Some("Ride The Lightning".to_string()),
            ..Default::default()
        });
        let ac = library.save(&ArtistRef {
            model_key: track.key.clone().unwrap(),
            artist_key: artist.key.clone().unwrap(),
            ..Default::default()
        });

        println!("{:?}", plugins.lyrics(&library, &track));
        println!("{:?}", plugins.metadata(&library, &track));
    }
}
