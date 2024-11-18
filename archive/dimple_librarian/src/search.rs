
use dimple_core::{
    db::Db, model::{Artist, Entity, Genre, Recording, ReleaseGroup}
};

use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};


pub fn db_search(db: &dyn Db, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
    const MAX_RESULTS_PER_TYPE: usize = 10;

    // TODO sort and filter by score

    let pattern = query.to_string();
    let matcher = SkimMatcherV2::default();
    let artists = db.list(&Artist::default().model(), &None)?
        .filter(move |artist| {
            let artist: Artist = artist.clone().into();
            matcher.fuzzy_match(&artist.name.clone().unwrap_or_default(), &pattern).is_some()
        })
        .take(MAX_RESULTS_PER_TYPE);
    let iter = artists;

    let pattern = query.to_string();
    let matcher = SkimMatcherV2::default();
    let release_groups = db.list(&ReleaseGroup::default().model(), &None)?
        .filter(move |rg| {
            let rg: ReleaseGroup = rg.clone().into();
            matcher.fuzzy_match(&rg.title.clone().unwrap_or_default(), &pattern).is_some()
        })
        .take(MAX_RESULTS_PER_TYPE);
    let iter = iter.chain(release_groups);

    let pattern = query.to_string();
    let matcher = SkimMatcherV2::default();
    let recordings = db.list(&Recording::default().model(), &None)?
        .filter(move |recording| {
            let recording: Recording = recording.clone().into();
            matcher.fuzzy_match(&recording.title.clone().unwrap_or_default(), &pattern).is_some()
        })
        .take(MAX_RESULTS_PER_TYPE);
    let iter = iter.chain(recordings);

    let pattern = query.to_string();
    let matcher = SkimMatcherV2::default();
    let genres = db.list(&Genre::default().model(), &None)?
        .filter(move |genre| {
            let genre: Genre = genre.clone().into();
            matcher.fuzzy_match(&genre.name.clone().unwrap_or_default(), &pattern).is_some()
        })
        .take(MAX_RESULTS_PER_TYPE);
    let iter = iter.chain(genres);

    Ok(Box::new(iter))
}

