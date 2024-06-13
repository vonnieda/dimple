use std::{collections::{HashMap, HashSet}, fs, path::Path, sync::{Arc, Mutex, RwLock}};

use dimple_core::{
    db::{Db, SqliteDb}, model::{Artist, Entity, Genre, Model, Picture, ReleaseGroup, Track}
};

use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use image::DynamicImage;

use crate::{merge::{self, Merge}, plugin::{NetworkMode, Plugin}};

pub fn db_search(db: &dyn Db, query: &str) -> Result<Box<dyn Iterator<Item = dimple_core::model::Model>>> {
    const MAX_RESULTS_PER_TYPE: usize = 25;

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
    let tracks = db.list(&Track::default().model(), &None)?
        .filter(move |track| {
            let track: Track = track.clone().into();
            matcher.fuzzy_match(&track.title.clone().unwrap_or_default(), &pattern).is_some()
        })
        .take(MAX_RESULTS_PER_TYPE);
    let iter = iter.chain(tracks);

    let pattern = query.to_string();
    let matcher = SkimMatcherV2::default();
    let release_groups = db.list(&Genre::default().model(), &None)?
        .filter(move |genre| {
            let genre: Genre = genre.clone().into();
            matcher.fuzzy_match(&genre.name.clone().unwrap_or_default(), &pattern).is_some()
        })
        .take(MAX_RESULTS_PER_TYPE);
    let iter = iter.chain(release_groups);

    Ok(Box::new(iter))
}

