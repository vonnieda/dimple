use crate::{library::Library, merge::CrdtRules, model::{Artist, Genre, LibraryModel, Model, ModelBasics as _, Release, Track}, plugins::plugin_host::PluginHost};

pub fn refresh_metadata(library: &Library, plugins: &PluginHost, model: &impl LibraryModel) {
    log::info!("refresh_metadata {:?} {:?}", model.type_name(), model.key());
    match model.type_name().as_str() {
        "Track" => {
            if let Some(track) = Track::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &track.clone()) {
                    library.save(&CrdtRules::merge(track, metadata));
                }
            }
        }
        "Artist" => {
            if let Some(artist) = Artist::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &artist.clone()) {
                    library.save(&CrdtRules::merge(artist, metadata));
                }
            }
        }
        "Release" => {
            if let Some(release) = Release::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &release.clone()) {
                    library.save(&CrdtRules::merge(release, metadata));
                }
            }
        }
        "Genre" => {
            if let Some(genre) = Genre::get(library, &model.key().clone().unwrap()) {
                if let Some(metadata) = plugins.metadata(library, &genre.clone()) {
                    library.save(&CrdtRules::merge(genre, metadata));
                }
            }
        }
        _ => todo!()
    }
}
