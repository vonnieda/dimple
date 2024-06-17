use std::collections::HashSet;

use dimple_core::{db::Db, model::{Artist, ArtistCredit, Entity, Genre, KnownIds, Medium, Model, Dimage, Recording, RecordingSource, Release, ReleaseGroup, Track}};

use crate::{equiv::Equivalent, matching};

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
            links: HashSet::merge(l.links, r.links),
            name: Option::merge(l.name, r.name),
            summary: Option::merge(l.summary, r.summary),
            genres: Vec::merge(l.genres, r.genres),
        }
    }
}

impl Merge for ReleaseGroup {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: HashSet::merge(l.links, r.links),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            first_release_date: Option::merge(l.first_release_date, r.first_release_date),
            primary_type: Option::merge(l.primary_type, r.primary_type),
            annotation: Option::merge(l.annotation, r.annotation),
            secondary_types: HashSet::merge(l.secondary_types, r.secondary_types),
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits),
            genres: Vec::merge(l.genres, r.genres),
        }
    }
}

impl Merge for Release {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: HashSet::merge(l.links, r.links),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            primary_type: Option::merge(l.primary_type, r.primary_type),
            barcode: Option::merge(l.barcode, r.barcode),
            country: Option::merge(l.country, r.country),
            date: Option::merge(l.date, r.date),
            packaging: Option::merge(l.packaging, r.packaging),
            status: Option::merge(l.status, r.status),
            quality: Option::merge(l.quality, r.quality),
            release_group: ReleaseGroup::merge(l.release_group, r.release_group),
            media: Vec::merge(l.media, r.media),
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits),
            genres: Vec::merge(l.genres, r.genres),
        }
    }
}

impl Merge for Medium {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disc_count: Option::merge(l.disc_count, r.disc_count),
            format: Option::merge(l.format, r.format),
            key: Option::merge(l.key, r.key),
            position: Option::merge(l.position, r.position),
            title: Option::merge(l.title, r.title),
            track_count: Option::merge(l.track_count, r.track_count),
            tracks: Vec::merge(l.tracks, r.tracks),
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
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits),
            genres: Vec::merge(l.genres, r.genres),
            recording: Recording::merge(l.recording, r.recording),
        }
    }
}

impl Merge for Recording {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: HashSet::merge(l.links, r.links),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            annotation: Option::merge(l.annotation, r.annotation),
            isrc: Option::merge(l.isrc, r.isrc),
            length: Option::merge(l.length, r.length),
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits),
            genres: Vec::merge(l.genres, r.genres),
        }
    }
}

impl Merge for Genre {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: KnownIds::merge(l.known_ids, r.known_ids),
            links: HashSet::merge(l.links, r.links),
            name: Option::merge(l.name, r.name),
            summary: Option::merge(l.summary, r.summary),
        }
    }
}

impl Merge for ArtistCredit {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: Option::merge(l.key, r.key),
            artist: Artist::merge(l.artist, r.artist),
            join_phrase: Option::merge(l.join_phrase, r.join_phrase),
            name: Option::merge(l.name, r.name),
        }
    }
}


// impl Merge for Dimage {
//     fn merge(l: Self, r: Self) -> Self {
//         Self {
//             key: Option::merge(l.key, r.key),
//             data: if l.data.len() >= r.data.len() { l.data } else { r.data },
//             // TODO this entire merge concept is weird here. Figure it out.
//             ..Default::default()
//         }
//     }
// }

impl Merge for KnownIds {
    fn merge(l: Self, r: Self) -> Self {
        KnownIds {
            musicbrainz_id: Option::merge(l.musicbrainz_id, r.musicbrainz_id),
            discogs_id: Option::merge(l.discogs_id, r.discogs_id),
            lastfm_id: Option::merge(l.lastfm_id, r.lastfm_id),
        }
    }
}

impl Merge for Model {
    fn merge(l: Self, r: Self) -> Self {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => Artist::merge(l.clone(), r.clone()).model(),
            (Model::Release(l), Model::Release(r)) => Release::merge(l.clone(), r.clone()).model(),
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => ReleaseGroup::merge(l.clone(), r.clone()).model(),
            (Model::Genre(l), Model::Genre(r)) => Genre::merge(l.clone(), r.clone()).model(),
            // (Model::Medium(l), Model::Medium(r)) => Medium::merge(l.clone(), r.clone()).model(),
            // (Model::Track(l), Model::Track(r)) => Track::merge(l.clone(), r.clone()).model(),
            // (Model::Dimage(l), Model::Dimage(r)) => Dimage::merge(l.clone(), r.clone()).model(),
            _ => todo!()
        }
    }
}

/// Combines two Vec of <Merge + Equivalent + Clone> and merges equivalent
/// results.
impl <T: Merge + Equivalent + Clone> Merge for Vec<T> {
    fn merge(l: Self, r: Self) -> Self {
        let mut result = l.clone();
    
        for b in r {
            let mut merged = false;
    
            for a in &mut result {
                if T::equivalent(a, &b) {
                    *a = T::merge(a.clone(), b.clone());
                    merged = true;
                    break;
                }
            }
    
            if !merged {
                result.push(b);
            }
        }
    
        result
    }
}

impl Merge for HashSet<String> {
    fn merge(l: Self, r: Self) -> Self {
        l.union(&r).cloned().collect()
    }
}

impl Merge for Option<u32> {
    fn merge(l: Self, r: Self) -> Self {
        match (l, r) {
            (Some(l), Some(r)) => if l >= r { Some(l) } else { Some(r) },
            (None, None) => None,
            (None, Some(r)) => Some(r),
            (Some(l), None) => Some(l),
        }
    }
}

impl Merge for Option<String> {
    fn merge(l: Self, r: Self) -> Self {
        match (l, r) {
            (Some(l), Some(r)) => if l.len() >= r.len() { Some(l) } else { Some(r) },
            (None, None) => None,
            (None, Some(r)) => Some(r),
            (Some(l), None) => Some(l),
        }
    }
}

fn db_merge_artist(db: &dyn Db, artist: &Artist) -> Option<Model> {
    let artist: Artist = db_merge_model(db, &artist.model(), &None)?.into();
    db_merge_genres(db, &artist.genres, &artist.model());
    Some(artist.model())
}

fn db_merge_release_group(db: &dyn Db, release_group: &ReleaseGroup) -> Option<Model> {
    let release_group: ReleaseGroup = 
        db_merge_model(db, &release_group.model(), &None)?.into();
    db_merge_genres(db, &release_group.genres, &release_group.model());
    db_merge_artist_credits(db, &release_group.artist_credits, &release_group.model());
    Some(release_group.model())
}

fn db_merge_release(db: &dyn Db, release: &Release) -> Option<Model> {
    let release: Release = 
        db_merge_model(db, &release.model(), &None)?.into();
    db_merge_genres(db, &release.genres, &release.model());
    db_merge_artist_credits(db, &release.artist_credits, &release.model());
    db_merge_media(db, &release.media, &release);
    Some(release.model())
}

fn db_merge_media(db: &dyn Db, media: &[Medium], release: &Release) {
    for medium in media {
        let medium = db_merge_medium(db, medium, release);
        lazy_link(db, &medium, &Some(release.model()));
    }
}

fn db_merge_medium(db: &dyn Db, medium: &Medium, release: &Release) -> Option<Model> {
    let medium: Medium = db_merge_model(db, &medium.model(), &Some(release.model()))?.into();
    db_merge_tracks(db, &medium.tracks, &medium);
    Some(medium.model())
}

fn db_merge_tracks(db: &dyn Db, tracks: &[Track], medium: &Medium) {
    for track in tracks {
        let track = db_merge_track(db, track, medium);
        lazy_link(db, &track, &Some(medium.model()));
    }
}

fn db_merge_track(db: &dyn Db, track: &Track, medium: &Medium) -> Option<Model> {
    db_merge_model(db, &track.model(), &Some(medium.model()))
}

fn db_merge_artist_credits(db: &dyn Db, artist_credits: &[ArtistCredit], related_to: &Model) {
    for artist_credit in artist_credits {
        // TODO temporary bypass artist credit for artist to get some testing
        // done
        let artist = db_merge_artist(db, &artist_credit.artist);
        lazy_link(db, &artist, &Some(related_to.clone()))
    }
}

fn db_merge_genres(db: &dyn Db, genres: &[Genre], related_to: &Model) {
    for genre in genres {
        let genre = db_merge_genre(db, genre);
        lazy_link(db, &genre, &Some(related_to.clone()))
    }
}

fn db_merge_genre(db: &dyn Db, genre: &Genre) -> Option<Model> {
    db_merge_model(db, &genre.model(), &None)
}

pub fn db_merge_model(db: &dyn Db, model: &Model, parent: &Option<Model>) -> Option<Model> {
    // TODO does this need to be merging parent as well?

    if let (Model::Dimage(_), None) = (model, parent) {
        panic!("Can't merge Dimage with no relation.");
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
        Model::Dimage(_p) => true,
        _ => todo!()
    }
}

pub fn merge(db: &dyn Db, model: &Model) -> Option<Model> {
    match model {
        // TODO I think I can move this logic into db_merge_model, and specifically
        // panic when asked to merge something without enough context. 
        // Like an Media without a Release or whatever.
        Model::Artist(artist) => db_merge_artist(db, artist),
        Model::Release(release) => db_merge_release(db, release),
        Model::ReleaseGroup(release_group) => db_merge_release_group(db, release_group),
        Model::Genre(genre) => db_merge_genre(db, genre),
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn basics() {
        assert!(Option::merge(None, Some("Lynyrd Skynrd".to_string())) == Some("Lynyrd Skynrd".to_string()));
        assert!(Option::merge(Some("Skynrd".to_string()), Some("Lynyrd Skynrd".to_string())) == Some("Lynyrd Skynrd".to_string()));
        assert!(Option::merge(Some(10), None) == Some(10));
        assert!(Option::merge(Some(10), Some(20)) == Some(20));
    }

    #[test]
    fn hashset() {
        let l = HashSet::from(["chicken".to_string(), "horse".to_string(), "dog".to_string()]);
        let r = HashSet::from(["monkey".to_string(), "pig".to_string(), "dog".to_string()]);
        let m = HashSet::merge(l, r);
        assert!(m.len() == 5);
        assert!(m.contains("chicken"));
        assert!(m.contains("horse"));
        assert!(m.contains("monkey"));
        assert!(m.contains("pig"));
        assert!(m.contains("dog"));
    }


    #[test]
    fn genre() {
        let m = Genre::merge(Genre {
            name: Some("dogrock".to_string()),
            ..Default::default()
        }, Genre {
            name: Some("catrock".to_string()),
            ..Default::default()
        });
        assert!(m.name == Some("dogrock".to_string()));

        let m = Genre::merge(Genre {
            name: Some("dogrock".to_string()),
            ..Default::default()
        }, Genre {
            name: Some("fishrock".to_string()),
            ..Default::default()
        });
        assert!(m.name == Some("fishrock".to_string()));
    }


    #[test]
    fn artist() {
        let l = Artist {
            key: Some("1234-1234-1234-1234".to_string()),
            name: Some("Ancient Fish".to_string()),
            genres: vec![
                Genre {
                    name: Some("fishjazz".to_string()),
                    summary: Some("Jazz, by fishes.".to_string()),
                    ..Default::default()
                },
                Genre {
                    name: Some("fishrock".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let r = Artist {
            key: Some("1234-1234-1234-1234".to_string()),
            name: Some("Ancient Fimsh".to_string()),
            genres: vec![
                Genre {
                    name: Some("fishjazz".to_string()),
                    known_ids: KnownIds { 
                        musicbrainz_id: Some("9999-9999-9999-9999".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Genre {
                    name: Some("fishtempo".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let m = Artist::merge(l, r);
        assert!(m.genres.len() == 3);
        // assert!(m.genres[0].)
    }

    #[test]
    fn release() {
        let l = Release {
            title: Some("Phood for Other Fish".to_string()),
            barcode: Some("123123123".to_string()),
            artist_credits: vec![
                ArtistCredit {
                    name: Some("Bob".to_string()),
                    join_phrase: Some("as".to_string()),
                    artist: Artist {
                        key: Some("72316492736498176349871234".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            genres: vec![
                Genre {
                    name: Some("fishjazz".to_string()),
                    known_ids: KnownIds { 
                        musicbrainz_id: Some("888-111-222-333".to_string()), 
                        discogs_id: None, 
                        lastfm_id: None, 
                    },
                    ..Default::default()
                }
            ],
            media: vec![
                Medium {
                    position: Some(1),
                    tracks: vec![
                        Track {
                            title: Some("Sizzlin'".to_string()),
                            recording: Recording {
                                isrc: Some("ASDASDASD".to_string()),
                                known_ids: KnownIds {
                                    discogs_id: Some("D10123123".to_string()),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let r = Release {
            title: Some("Phood for Other Fish".to_string()),
            disambiguation: Some("Second press".to_string()),
            artist_credits: vec![
                ArtistCredit {
                    name: Some("Bob".to_string()),
                    join_phrase: Some("as".to_string()),
                    artist: Artist {
                        key: Some("72316492736498176349871234".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            genres: vec![
                Genre {
                    name: Some("fishjazz".to_string()),
                    ..Default::default()
                }
            ],
            country: Some("us".to_string()),
            media: vec![
                Medium {
                    position: Some(1),
                    tracks: vec![
                        Track {
                            title: Some("Sizzlin'".to_string()),
                            recording: Recording {
                                isrc: Some("ASDASDASD".to_string()),
                                known_ids: KnownIds {
                                    musicbrainz_id: Some("98123-2342345-2345-234-5345".to_string()),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Track {
                            title: Some("Into the Frying Pan".to_string()),
                            recording: Recording {
                                isrc: Some("FISHFISH".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let m = Release::merge(l, r);
        // dbg!(&m);
        assert!(m.media.get(0).unwrap().tracks.get(0).unwrap().recording.known_ids.discogs_id == Some("D10123123".to_string()));
        assert!(m.media.get(0).unwrap().tracks.get(1).unwrap().title == Some("Into the Frying Pan".to_string()));
    }
}

