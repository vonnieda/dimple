use dimple_core::{library::Library, model::{Artist, DimageRef, DimpleEntity, Genre, Playlist, Release, Track}, plugins::plugins::Plugins};
use slint::{ComponentHandle as _, Model as _, ModelRc, VecModel, Weak};

use crate::ui::{images::dynamic_to_slint, AppWindow, LazyImageLoader, LazyImageModel};

pub fn init_lazy_image_loader(ui: &AppWindow, library: &Library, plugins: &Plugins) {
    // TODO replace with (and create) HashMapModel or something so we can use
    // keys.
    ui.global::<LazyImageLoader>().set_images(ModelRc::new(VecModel::<LazyImageModel>::default()));
    let ui_weak = ui.as_weak();
    let library = library.clone();
    let plugins = plugins.clone();
    ui.global::<LazyImageLoader>().on_load(move |images, key, width, height| {
        let ui_weak = ui_weak.clone();
        // Downcast is okay because we explicitly set to VecModel above.
        let images = images.as_any().downcast_ref::<VecModel<LazyImageModel>>().unwrap();
        let index: usize = images.iter().enumerate()
            .find_map(|(index, model)| {
                if model.key == key {
                    Some(index)
                }
                else {
                    None
                }
            })
            .unwrap_or_else(|| {
                images.push(LazyImageModel {
                    key: key.clone(),
                    loaded: false,
                    ..Default::default()
                });
                let index = images.row_count() - 1;
                async_load(ui_weak, &library, &plugins, &key, index);
                index
            });
        index as i32
    });
}

pub fn async_load(app_weak: Weak<AppWindow>, library: &Library, plugins: &Plugins, key: &str, index: usize) {
    if key.is_empty() {
        return
    }
    let key = key.to_string();
    let library = library.clone();
    let plugins = plugins.clone();
    std::thread::spawn(move || {
        // TODO hax
        let entity: Option<DimpleEntity> = library.get::<Artist>(&key).map(DimpleEntity::from)
            .or_else(|| library.get::<Release>(&key).map(DimpleEntity::from))
            .or_else(|| library.get::<Genre>(&key).map(DimpleEntity::from))
            .or_else(|| library.get::<Track>(&key).map(DimpleEntity::from))
            .or_else(|| library.get::<Playlist>(&key).map(DimpleEntity::from))
            ;
        if entity.is_none() {
            log::warn!("no entity found for key '{key}'");
            return
        }
        let entity = entity.unwrap();

        let image = library.image(&entity)
            .or_else(|| {
                if let Some(plugin_image) = plugins.image(&library, &entity).get(0) {
                    let dimage = library.db.transaction(|txn| {
                        let dimage = txn.save(plugin_image)?;
                        DimageRef::attach(txn, &dimage, &Some(key.clone()))?;
                        Ok(dimage)
                    }).unwrap();
                    Some(dimage.get_image())
                }
                else {
                    None
                }
            });
        if image.is_none() {
            log::warn!("no image found for entity {} '{}'", entity.type_name(), key);
            return
        }
        let image = image.unwrap();
        app_weak.upgrade_in_event_loop(move |ui| {
            let image = dynamic_to_slint(&image);
            let images = ui.global::<LazyImageLoader>().get_images();
            let mut model = images.row_data(index).unwrap();
            model.image = image;
            model.loaded = true;
            images.set_row_data(index, model);
        }).unwrap();
    });
}