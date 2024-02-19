use dimple_core::{collection::Collection, model::Entities};
use image::DynamicImage;

use crate::librarian::Librarian;

/// Generate some kind of cool artwork for the entity to be used as a
/// default. It can use data from the library to create the image.
pub fn generate_masterpiece(librarian: &Librarian, entity: &Entities, width: u32, 
    height: u32) -> DynamicImage {

    // https://stackoverflow.com/questions/76741218/in-slint-how-do-i-render-a-self-drawn-image
    // http://ia802908.us.archive.org/35/items/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c/mbid-8d4a5efd-7c87-487d-97e7-30e5fc6b9a8c-25647975198.jpg

    match entity {
        Entities::Artist(a) => {
            for rg in a.release_groups(librarian) {
                if let Some(dyn_image) = librarian.image(&rg.entity()) {
                    log::warn!("Generated artist image for {} using release group {}", 
                        a.name.clone().unwrap_or_default(), 
                        rg.title.clone().unwrap_or_default());
                    return dyn_image;
                }
            }
        },
        Entities::ReleaseGroup(r) => {

        },
        Entities::Release(r) => {

        },
        _ => {}
    }

    DynamicImage::new_rgb8(width, height)
}

