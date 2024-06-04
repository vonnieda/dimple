use dimple_core::{db::Db, model::{Entity, Model, ReleaseGroup}};

pub fn find_matching_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
    match model {
        Model::ReleaseGroup(release_group) => find_release_group(db, release_group),
        _ => db.list(&model, parent).unwrap().find(|model_opt| compare_models(&model, model_opt))
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
    None
}

fn is_some_and_equal(l: &Option<String>, r: &Option<String>) -> bool {
    l.is_some() && l == r
}

fn compare_models(l: &Model, r: &Model) -> bool {
    match (l, r) {
        (Model::Artist(l), Model::Artist(r)) => {
            (l.name.is_some() && l.name == r.name && l.disambiguation == r.disambiguation)
            || (l.known_ids.musicbrainz_id.is_some() && l.known_ids.musicbrainz_id == r.known_ids.musicbrainz_id)
        },
        (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => {
            (l.title.is_some() && l.title == r.title)
            || (l.known_ids.musicbrainz_id.is_some() && l.known_ids.musicbrainz_id == r.known_ids.musicbrainz_id)
        },
        (Model::Release(l), Model::Release(r)) => {
            l.title.is_some() && l.title == r.title
        },
        (Model::Medium(l), Model::Medium(r)) => {
            l.position == r.position
        },
        (Model::Track(l), Model::Track(r)) => {
            l.title.is_some() && l.title == r.title
        },
        (Model::Genre(l), Model::Genre(r)) => {
            l.name.is_some() && l.name == r.name
        },
        (Model::ArtistCredit(l), Model::ArtistCredit(r)) => {
            l.name.is_some() && l.name == r.name
        },
        _ => todo!()
    }
}

