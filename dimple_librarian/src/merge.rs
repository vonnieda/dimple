use std::collections::HashSet;

use dimple_core::{db::Db, model::{Artist, ArtistCredit, Entity, Genre, KnownIds, Medium, Model, Picture, Recording, RecordingSource, Release, ReleaseGroup, Track}};

use crate::matching;

pub trait Merge {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn merge(l: Self, r: Self) -> Self;
}

impl Merge for Artist {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            country: Option::merge(l.country, r.country),
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: l.links.union(&r.links).cloned().collect(),
            name: Option::merge(l.name, r.name),
            summary: Option::merge(l.summary, r.summary),
            genres: l.genres.iter().chain(r.genres.iter()).cloned().collect::<HashSet<Genre>>().into_iter().collect(),
        }
    }
}

// TODO most are unfinished - still experimenting
impl Merge for ReleaseGroup {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: l.links.union(&r.links).cloned().collect(),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            first_release_date: Option::merge(l.first_release_date, r.first_release_date),
            primary_type: Option::merge(l.primary_type, r.primary_type),
            annotation: Option::merge(l.annotation, r.annotation),
            genres: l.genres.iter().chain(r.genres.iter()).cloned().collect::<HashSet<Genre>>().into_iter().collect(),
            artist_credits: l.artist_credits.iter().chain(r.artist_credits.iter()).cloned().collect::<HashSet<ArtistCredit>>().into_iter().collect(),
        }
    }
}

impl Merge for Release {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: l.links.union(&r.links).cloned().collect(),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            // first_release_date: Option::merge(l.first_release_date, r.first_release_date),
            // primary_type: Option::merge(l.primary_type, r.primary_type),
            // TODO
            ..Default::default()
        }
    }
}

impl Merge for Medium {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disc_count: l.disc_count.or(r.disc_count),
            format: l.format.or(r.format),
            key: l.key.or(r.key),
            position: l.position.or(r.position),
            title: l.title.or(r.title),
            track_count: l.track_count.or(r.track_count),
            ..Default::default()
        }
    }
}

impl Merge for Track {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            title: Option::merge(l.title, r.title),
            length: Option::merge(l.length, r.length),
            number: Option::merge(l.number, r.number),
            position: Option::merge(l.position, r.position),
            ..Default::default()
        }
    }
}

impl Merge for Genre {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: l.links.union(&r.links).cloned().collect(),
            name: Option::merge(l.name, r.name),
            summary: Option::merge(l.summary, r.summary),
        }
    }
}

impl Merge for Recording {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: l.links.union(&r.links).cloned().collect(),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            annotation: Option::merge(l.annotation, r.annotation),
            // TODO
            ..Default::default()
        }
    }
}

impl Merge for Picture {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: Option::merge(l.key, r.key),
            data: if l.data.len() >= r.data.len() { l.data } else { r.data },
        }
    }
}

impl Merge for Model {
    fn merge(l: Self, r: Self) -> Self {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => Artist::merge(l.clone(), r.clone()).model(),
            (Model::Genre(l), Model::Genre(r)) => Genre::merge(l.clone(), r.clone()).model(),
            (Model::Medium(l), Model::Medium(r)) => Medium::merge(l.clone(), r.clone()).model(),
            (Model::Release(l), Model::Release(r)) => Release::merge(l.clone(), r.clone()).model(),
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => ReleaseGroup::merge(l.clone(), r.clone()).model(),
            (Model::Track(l), Model::Track(r)) => Track::merge(l.clone(), r.clone()).model(),
            (Model::Picture(l), Model::Picture(r)) => Picture::merge(l.clone(), r.clone()).model(),
            _ => todo!()
        }
    }
}

impl Merge for Option<u32> {
    fn merge(l: Self, r: Self) -> Self {
        l.or(r)
    }
}

impl Merge for Option<String> {
    fn merge(l: Self, r: Self) -> Self {
        l.or(r)
    }
}

impl Merge for KnownIds {
    fn merge(l: Self, r: Self) -> Self {
        KnownIds {
            musicbrainz_id: Option::merge(l.musicbrainz_id, r.musicbrainz_id),
            discogs_id: Option::merge(l.discogs_id, r.discogs_id),
            lastfm_id: Option::merge(l.lastfm_id, r.lastfm_id),
        }
    }
}

fn merge_artist(db: &dyn Db, artist: &Artist) -> Option<Model> {
    let artist: Artist = db_merge_model(db, &artist.model(), &None)?.into();
    for genre in &artist.genres {
        let genre = merge_genre(db, genre);
        lazy_link(db, &genre, &Some(artist.model()))
    }
    Some(artist.model())
}

fn merge_release_group(db: &dyn Db, release_group: &ReleaseGroup) -> Option<Model> {
    let release_group: ReleaseGroup = 
        db_merge_model(db, &release_group.model(), &None)?.into();

    for genre in &release_group.genres {
        let genre = merge_genre(db, genre);
        lazy_link(db, &genre, &Some(release_group.model()))
    }

    // TODO temporary bypass artist credit for artist to get some testing
    // done
    for artist_credit in &release_group.artist_credits {
        let artist = merge_artist(db, &artist_credit.artist);
        lazy_link(db, &artist, &Some(release_group.model()))
    }

    Some(release_group.model())
}

/// get, merge, update the release properties
/// for each genre
///     merge(genre, release)
///     link(release, genre)
/// for each artist_credit
///     merge(artist_credit)
///     link(release, artist_credit)
///     link(release, artist)
/// for each medium
///     for each track
///         merge(release, medium, track)
fn merge_release(db: &dyn Db, release: &Release) -> Option<Model> {
    db_merge_model(db, &release.model(), &None)
}

fn merge_genre(db: &dyn Db, genre: &Genre) -> Option<Model> {
    db_merge_model(db, &genre.model(), &None)
}

pub fn db_merge_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
    // TODO does this need to be merging parent as well?

    if let (Model::Picture(_), None) = (model, parent) {
        panic!("Can't merge Picture with no relation.");
    }

    // find a matching model to the specified, merge, save
    let matching = matching::find_matching_model(db, model, parent);
    if let Some(matching) = matching {
        let merged = Model::merge(model.clone(), matching);
        return Some(db.insert(&merged).unwrap())
    }
    // if not, insert the new one and link it to the parent
    else {
        if model_valid(model) {
            let model = Some(db.insert(model).unwrap());
            lazy_link(db, &model, parent);
            return model
        }
    }
    None
}

/// Links the two Models if they are both Some. Reduces boilerplate.
fn lazy_link(db: &dyn Db, l: &Option<Model>, r: &Option<Model>) {
    if l.is_some() && r.is_some() {
        db.link(&l.clone().unwrap(), &r.clone().unwrap()).unwrap()
    }
}

fn model_valid(model: &Model) -> bool {
    match model {
        Model::Artist(a) => a.name.is_some() || a.known_ids.musicbrainz_id.is_some(),
        Model::Genre(g) => g.name.is_some(),
        Model::Medium(_m) => true,
        Model::Release(r) => r.title.is_some(),
        Model::ReleaseGroup(rg) => rg.title.is_some(),
        Model::Track(t) => t.title.is_some(),
        Model::Picture(p) => true,
        _ => todo!()
    }
}

pub fn merge(db: &dyn Db, model: &Model) -> Option<Model> {
    match model {
        // TODO I think I can move this logic into db_merge_model, and specifically
        // panic when asked to merge something without enough context. 
        // Like an Media without a Release or whatever.
        Model::Artist(artist) => merge_artist(db, artist),
        Model::Release(release) => merge_release(db, release),
        Model::ReleaseGroup(release_group) => merge_release_group(db, release_group),
        Model::Genre(genre) => merge_genre(db, genre),
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artist_merge() {
        let a1 = Artist {
            name: Some("Sorta Charger".to_string()),
            country: Some("us".to_string()),
            disambiguation: None,
            known_ids: KnownIds {
                musicbrainz_id: Some("123-123-123-123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let a2 = Artist {
            name: Some("Sorta Charger".to_string()),
            ..Default::default()
        };

        let a3 = Artist {
            name: Some("sorta charger".to_string()),
            known_ids: KnownIds {
                musicbrainz_id: Some("123-123-123-123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let a4 = Artist {
            name: Some("slorta charger".to_string()),
            known_ids: KnownIds {
                musicbrainz_id: Some("123-123-123-123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let a5 = Artist {
            name: Some("Sorta Charger".to_string()),
            country: Some("us".to_string()),
            disambiguation: Some("the other one".to_string()),
            known_ids: KnownIds {
                musicbrainz_id: Some("123-123-123-123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        // dbg!(Artist::mergability(&a1, &a2));
        // dbg!(Artist::mergability(&a1, &a3));
        // dbg!(Artist::mergability(&a1, &a4));
        // dbg!(Artist::mergability(&a1, &a5));
    }
}

