use dimple_core::{db::Db, model::{Entity, Model, ReleaseGroup}};

use crate::equivalent::Equivalent;

// TODO revisit this entirely.

/// Finds a model that matches, and can be merged with, the specified model.
/// When related_to is supplied the search for a compatible model is restricted
/// to relations of the related_to model, otherwise all objects are searched.
/// 
/// TODO this currently does a list all and find via equivalent for most
/// objects, but this is where I can add some real actual database queries
/// to speed things up.
pub fn find_matching_model(db: &dyn Db, model: &Model, related_to: &Option<Model>) -> Option<Model> {
    if model.entity().key().is_some() {
        if let Ok(Some(model)) = db.get(model) {
            return Some(model);
        }
    }
    match model {
        Model::ReleaseGroup(release_group) => find_release_group(db, release_group),
        _ => db.list(&model, related_to).unwrap().find(|model_opt| Model::equivalent(&model, model_opt))
    }
}


fn find_release_group(db: &dyn Db, release_group: &ReleaseGroup) -> Option<Model> {
    // find by key
    if let Some(_key) = &release_group.key {
        return db.get(&release_group.model()).unwrap()
    }

    // find by known id
    let matched = db.list(&release_group.model(), &None).unwrap()
        .map(Into::<ReleaseGroup>::into)
        .find(|opt| {
            is_some_and_equal(&release_group.known_ids.musicbrainz_id, &opt.known_ids.musicbrainz_id)
        });
    if let Some(matched) = matched {
        return Some(matched.model())
    }

    // find by artist + title
    // TODO 
    None
}

fn is_some_and_equal(l: &Option<String>, r: &Option<String>) -> bool {
    l.is_some() && l == r
}

