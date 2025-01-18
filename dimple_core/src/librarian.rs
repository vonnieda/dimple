use crate::{library::Library, merge::CrdtRules, model::{Model, ModelBasics as _, Track}, plugins::plugin_host::PluginHost};

pub fn refresh_metadata(library: &Library, plugins: &PluginHost, model: &impl Model) {
    log::info!("refresh_metadata {:?} {:?}", model.type_name(), model.key());
    match model.type_name().as_str() {
        "Track" => {
            if let Some(track) = Track::get(library, &model.key().clone().unwrap()) {
                if let Ok(Some(metadata)) = plugins.metadata(library, &track) {
                    library.save(&CrdtRules::merge(track, metadata));
                }
            }
        }
        _ => todo!()
    }
}
