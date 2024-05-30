use std::collections::HashSet;

use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, RecordingSource, Release, ReleaseGroup, Track};

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