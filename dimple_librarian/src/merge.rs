use dimple_core::model::{Artist, KnownIds, Recording, RecordingSource, Release, ReleaseGroup, Track};

pub trait Merge {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn merge(l: Self, r: Self) -> Self;

    /// -INFINITY <= conflict <= 0.0 no match 1.0 <= +INFINITY
    /// TODO this should probably just be a Result<f32> where Err is conflict
    /// and Ok() gives the score.
    fn mergability(l: &Self, r: &Self) -> f32;
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
        }
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        let mut score: f32 = 0.0;

        score += Option::mergability(&l.key, &r.key);
        score += KnownIds::mergability(&l.known_ids, &r.known_ids);

        let mut name_score = Option::mergability(&l.name, &r.name);
        if name_score >= 1.0 {
            name_score += Option::mergability(&l.country, &r.country);
            name_score += Option::mergability(&l.disambiguation, &r.disambiguation);
        }
        score += name_score;

        score
    }
}

// TODO Note maybe going a different direction than this. The two below are not
// finished.

// impl Merge for ReleaseGroup {
//     fn merge(l: Self, r: Self) -> Self {
//         Self {
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation),
//             key: Option::merge(l.key, r.key),
//             known_ids: l.known_ids.union(&r.known_ids).cloned().collect(),
//             links: l.links.union(&r.links).cloned().collect(),
//             title: Option::merge(l.title, r.title),
//             summary: Option::merge(l.summary, r.summary),
//             first_release_date: Option::merge(l.first_release_date, r.first_release_date),
//             primary_type: Option::merge(l.primary_type, r.primary_type),
//         }
//     }

//     fn mergability(l: &Self, r: &Self) -> f32 {
//         let mut score: f32 = 0.0;

//         score += Option::mergability(&l.key, &r.key);
//         // score += KnownIds::mergability(&l.known_ids, &r.known_ids);

//         // TODO need more than title! age old problem.
//         let mut name_score = Option::mergability(&l.title, &r.title);
//         if name_score >= 1.0 {
//             name_score += Option::mergability(&l.disambiguation, &r.disambiguation);
//         }
//         score += name_score;

//         score
//     }
// }

impl Merge for Release {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: l.known_ids.union(&r.known_ids).cloned().collect(),
            links: l.links.union(&r.links).cloned().collect(),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            // first_release_date: Option::merge(l.first_release_date, r.first_release_date),
            // primary_type: Option::merge(l.primary_type, r.primary_type),
            // TODO
            ..Default::default()
        }
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        let mut score: f32 = 0.0;

        score += Option::mergability(&l.key, &r.key);
        // score += KnownIds::mergability(&l.known_ids, &r.known_ids);

        // TODO need more than title! age old problem.
        let mut name_score = Option::mergability(&l.title, &r.title);
        if name_score >= 1.0 {
            name_score += Option::mergability(&l.disambiguation, &r.disambiguation);
        }
        score += name_score;

        score
    }
}

impl Merge for Recording {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation),
            key: Option::merge(l.key, r.key),
            known_ids: l.known_ids.union(&r.known_ids).cloned().collect(),
            links: l.links.union(&r.links).cloned().collect(),
            title: Option::merge(l.title, r.title),
            summary: Option::merge(l.summary, r.summary),
            annotation: Option::merge(l.annotation, r.annotation),
            // TODO
            ..Default::default()
        }
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        todo!()
    }
}

impl Merge for Track {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: Option::merge(l.key, r.key),
            known_ids: l.known_ids.union(&r.known_ids).cloned().collect(),
            title: Option::merge(l.title, r.title),
            length: Option::merge(l.length, r.length),
            number: Option::merge(l.number, r.number),
            position: Option::merge(l.position, r.position),
        }
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        todo!()
    }
}

impl Merge for Option<u32> {
    fn merge(l: Self, r: Self) -> Self {
        l.or(r)
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        todo!()
    }
}

impl Merge for Option<String> {
    fn merge(l: Self, r: Self) -> Self {
        l.or(r)
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        match (l, r) {
            (Some(l), Some(r)) => {
                if l.to_lowercase() == r.to_lowercase() {
                    1.0
                }
                else {
                    f32::NEG_INFINITY
                }
            },
            _ => 0.0
        }
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

    fn mergability(l: &Self, r: &Self) -> f32 {
        Option::mergability(&l.musicbrainz_id, &r.musicbrainz_id)
        + Option::mergability(&l.discogs_id, &r.discogs_id)
        + Option::mergability(&l.lastfm_id, &r.lastfm_id)
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

        dbg!(Artist::mergability(&a1, &a2));
        dbg!(Artist::mergability(&a1, &a3));
        dbg!(Artist::mergability(&a1, &a4));
        dbg!(Artist::mergability(&a1, &a5));
    }
}