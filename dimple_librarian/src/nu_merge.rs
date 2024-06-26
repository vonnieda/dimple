use std::collections::HashSet;

use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, Release, ReleaseGroup, Track};

/// Exploring a version of merge that is fallible, which indicates a merge
/// conflict. By bubbling these up it's easy to recurse, and merge a tree.
/// Haha, I'm a dummy. I could implement this for the JSON types and call
/// it a day, right?
pub trait NuMerge {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized;
}

impl NuMerge for Artist {
    fn nu_merge(l: Self, r: Self) -> Option<Self> {
        Some(Self {
            country: Option::nu_merge(l.country, r.country)?,
            disambiguation: Option::nu_merge(l.disambiguation, r.disambiguation)?,
            key: Option::nu_merge(l.key, r.key)?,
            known_ids: KnownIds::nu_merge(l.known_ids, r.known_ids)?,
            links: HashSet::nu_merge(l.links, r.links)?,
            name: Option::nu_merge(l.name, r.name)?,
            summary: Option::nu_merge(l.summary, r.summary)?,
            genres: Vec::nu_merge(l.genres, r.genres)?,
            // TODO assumes r is the new value
            saved: r.saved,
        })
    }
}

impl NuMerge for Genre {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            disambiguation: Option::nu_merge(l.disambiguation, r.disambiguation)?,
            key: Option::nu_merge(l.key, r.key)?,
            known_ids: KnownIds::nu_merge(l.known_ids, r.known_ids)?,
            links: HashSet::nu_merge(l.links, r.links)?,
            name: Option::nu_merge(l.name, r.name)?,
            summary: Option::nu_merge(l.summary, r.summary)?,
        })
    }
}

impl NuMerge for ArtistCredit {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::nu_merge(l.key, r.key)?,
            name: Option::nu_merge(l.name, r.name)?,
            join_phrase: Option::nu_merge(l.join_phrase, r.join_phrase)?,
            artist: Artist::nu_merge(l.artist, r.artist)?,
        })
    }
}

impl NuMerge for Medium {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::nu_merge(l.key, r.key)?,
            title: Option::nu_merge(l.title, r.title)?,
            disc_count: Option::nu_merge(l.disc_count, r.disc_count)?,
            format: Option::nu_merge(l.format, r.format)?,
            position: Option::nu_merge(l.position, r.position)?,
            track_count: Option::nu_merge(l.track_count, r.track_count)?,
            tracks: Vec::nu_merge(l.tracks, r.tracks)?,
        })
    }
}

impl NuMerge for Track {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::nu_merge(l.key, r.key)?,
            title: Option::nu_merge(l.title, r.title)?,
            position: Option::nu_merge(l.position, r.position)?,
            artist_credits: Vec::nu_merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::nu_merge(l.genres, r.genres)?,
            known_ids: KnownIds::nu_merge(l.known_ids, r.known_ids)?,
            length: Option::nu_merge(l.length, r.length)?,
            number: Option::nu_merge(l.number, r.number)?,
            recording: Recording::nu_merge(l.recording, r.recording)?,
        })
    }
}

impl NuMerge for ReleaseGroup {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::nu_merge(l.key, r.key)?,
            title: Option::nu_merge(l.title, r.title)?,
            artist_credits: Vec::nu_merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::nu_merge(l.genres, r.genres)?,
            known_ids: KnownIds::nu_merge(l.known_ids, r.known_ids)?,
            disambiguation: Option::nu_merge(l.disambiguation, r.disambiguation)?,
            links: HashSet::nu_merge(l.links, r.links)?,
            summary: Option::nu_merge(l.summary, r.summary)?,
            annotation: Option::nu_merge(l.annotation, r.annotation)?,            
            first_release_date: Option::nu_merge(l.first_release_date, r.first_release_date)?,
            primary_type: Option::nu_merge(l.primary_type, r.primary_type)?,
            secondary_types: HashSet::nu_merge(l.secondary_types, r.secondary_types)?,
        })
    }
}

impl NuMerge for Release {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::nu_merge(l.key, r.key)?,
            title: Option::nu_merge(l.title, r.title)?,
            artist_credits: Vec::nu_merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::nu_merge(l.genres, r.genres)?,
            known_ids: KnownIds::nu_merge(l.known_ids, r.known_ids)?,
            disambiguation: Option::nu_merge(l.disambiguation, r.disambiguation)?,
            links: HashSet::nu_merge(l.links, r.links)?,
            summary: Option::nu_merge(l.summary, r.summary)?,
            primary_type: Option::nu_merge(l.primary_type, r.primary_type)?,
            barcode: Option::nu_merge(l.barcode, r.barcode)?,
            country: Option::nu_merge(l.country, r.country)?,
            date: Option::nu_merge(l.date, r.date)?,
            packaging: Option::nu_merge(l.packaging, r.packaging)?,
            quality: Option::nu_merge(l.quality, r.quality)?,
            status: Option::nu_merge(l.status, r.status)?,
            media: Vec::nu_merge(l.media, r.media)?,
            release_group: ReleaseGroup::nu_merge(l.release_group, r.release_group)?,
        })
    }
}

impl NuMerge for Recording {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::nu_merge(l.key, r.key)?,
            title: Option::nu_merge(l.title, r.title)?,
            artist_credits: Vec::nu_merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::nu_merge(l.genres, r.genres)?,
            known_ids: KnownIds::nu_merge(l.known_ids, r.known_ids)?,
            length: Option::nu_merge(l.length, r.length)?,
            disambiguation: Option::nu_merge(l.disambiguation, r.disambiguation)?,
            links: HashSet::nu_merge(l.links, r.links)?,
            summary: Option::nu_merge(l.summary, r.summary)?,
            annotation: Option::nu_merge(l.annotation, r.annotation)?,
            isrc: Option::nu_merge(l.isrc, r.isrc)?,
        })
    }
}

impl NuMerge for String {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        if l == r {
            Some(l)
        }
        else {
            None
        }
    }
}

impl NuMerge for u32 {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        if l == r {
            Some(l)
        }
        else {
            None
        }
    }
}

impl <T: NuMerge> NuMerge for Option<T> {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        match (l, r) {
            (Some(l), Some(r)) => Some(Some(NuMerge::nu_merge(l, r)?)),
            (None, None) => Some(None),
            (None, Some(r)) => Some(Some(r)),
            (Some(l), None) => Some(Some(l)),
        }
    }
}

impl NuMerge for KnownIds {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(KnownIds {
            musicbrainz_id: Option::nu_merge(l.musicbrainz_id, r.musicbrainz_id)?,
            discogs_id: Option::nu_merge(l.discogs_id, r.discogs_id)?,
            lastfm_id: Option::nu_merge(l.lastfm_id, r.lastfm_id)?,
        })
    }
}

impl NuMerge for HashSet<String> {
    fn nu_merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(l.union(&r).cloned().collect())
    }
}

impl <T: NuMerge + Clone> NuMerge for Vec<T> {
    fn nu_merge(l: Self, r: Self) -> Option<Self> {
        let mut result = l.clone();
    
        for b in r {
            let mut merged = false;
    
            for a in &mut result {
                let m = T::nu_merge(a.clone(), b.clone());
                if m.is_some() {
                    *a = m.unwrap();
                    merged = true;
                    break;
                }
            }
    
            if !merged {
                result.push(b);
            }
        }
    
        Some(result)
    }
}

mod tests {
    use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, Release, Track};

    use crate::nu_merge::NuMerge;

    #[test]
    fn basics() {
        // assert!(NuMerge::nu_merge(None, None) == Some(None));
        assert!(NuMerge::nu_merge(Some("a".to_string()), None) == Some(Some("a".to_string())));
        assert!(NuMerge::nu_merge(None, Some("a".to_string())) == Some(Some("a".to_string())));
        assert!(NuMerge::nu_merge(Some("a".to_string()), Some("a".to_string())) == Some(Some("a".to_string())));
        assert!(NuMerge::nu_merge(Some("a".to_string()), Some("b".to_string())) == None);
    }

    #[test]
    fn artist1() {
        let l = Artist {
            name: Some("a".to_string()),
            ..Default::default()
        };
        let r = Artist {
            name: Some("b".to_string()),
            ..Default::default()
        };
        let m = NuMerge::nu_merge(l, r);
        assert!(m == None);
    }

    #[test]
    fn artist2() {
        let l = Artist {
            name: Some("a".to_string()),
            ..Default::default()
        };
        let r = Artist {
            name: Some("a".to_string()),
            ..Default::default()
        };
        let m = NuMerge::nu_merge(l.clone(), r.clone()).unwrap();
        assert!(l == m);
        assert!(r == m);
    }

    #[test]
    fn artist3() {
        let l = Artist {
            name: Some("a".to_string()),
            disambiguation: Some("d".to_string()),
            ..Default::default()
        };
        let r = Artist {
            name: Some("a".to_string()),
            ..Default::default()
        };
        let m = NuMerge::nu_merge(l.clone(), r.clone()).unwrap();
        assert!(l == m);
        assert!(r != m);
    }

    #[test]
    fn artist4() {
        let l = Artist {
            name: Some("a".to_string()),
            disambiguation: Some("d".to_string()),
            ..Default::default()
        };
        let r = Artist {
            name: Some("a".to_string()),
            summary: Some("s".to_string()),
            ..Default::default()
        };
        let m = NuMerge::nu_merge(l.clone(), r.clone()).unwrap();
        assert!(l != m);
        assert!(r != m);
        assert!(m.disambiguation == Some("d".to_string()));
        assert!(m.summary == Some("s".to_string()));
    }

    #[test]
    fn artist5() {
        let l = Artist {
            key: Some("k1".to_string()),
            name: Some("a".to_string()),
            disambiguation: Some("d".to_string()),
            ..Default::default()
        };
        let r = Artist {
            key: Some("k2".to_string()),
            name: Some("a".to_string()),
            summary: Some("s".to_string()),
            ..Default::default()
        };
        let m = NuMerge::nu_merge(l.clone(), r.clone());
        // merge conflict between key k1 and key k2.
        assert!(m.is_none());
    }

    #[test]
    fn genre() {
        let l = vec![
            Genre {
                name: Some("a".to_string()),
                known_ids: KnownIds {
                    musicbrainz_id: Some("23e4d287-9ddd-4fbc-9e57-74b92e269733".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            Genre {
                name: Some("b".to_string()),
                ..Default::default()
            },
        ];
        let r = vec![
            Genre {
                name: Some("a".to_string()),
                known_ids: KnownIds {
                    musicbrainz_id: Some("fad7bccc-9b31-49f0-82a6-0fceaaa40187".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            Genre {
                name: Some("b".to_string()),
                ..Default::default()
            },
            Genre {
                name: Some("c".to_string()),
                ..Default::default()
            },
        ];
        let m = NuMerge::nu_merge(l.clone(), r.clone()).unwrap();
        assert!(m.len() == 4);
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
        let m = Release::nu_merge(l, r).unwrap();
        dbg!(&m);
        assert!(m.media.get(0).unwrap().tracks.get(0).unwrap().recording.known_ids.discogs_id == Some("D10123123".to_string()));
        assert!(m.media.get(0).unwrap().tracks.get(1).unwrap().title == Some("Into the Frying Pan".to_string()));
    }

    #[test]
    fn test2() {
        let l = Release {
            key: Some("6238582b-b14b-4bab-b28c-a4f9f7d606de".to_string()),
            title: Some("Release 1".to_string()),
            ..Default::default()
        };
        let r = Release {
            key: Some("28d45028-dad5-45f4-9ea1-0158ccffd2cf".to_string()),
            title: Some("Release 1".to_string()),
            ..Default::default()
        };
        let m = NuMerge::nu_merge(l, r);
        dbg!(m);
    }
}


