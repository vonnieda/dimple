use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, Release, Track};
use dimple_librarian::{merge::Merge, mock::create_release};

#[test]
fn basics() {
    assert!(Merge::merge(Some("a".to_string()), None) == Some(Some("a".to_string())));
    assert!(Merge::merge(None, Some("a".to_string())) == Some(Some("a".to_string())));
    assert!(Merge::merge(Some("a".to_string()), Some("a".to_string())) == Some(Some("a".to_string())));
    assert!(Merge::merge(Some("a".to_string()), Some("b".to_string())) == None);
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
    let m = Merge::merge(l, r);
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
    let m = Merge::merge(l.clone(), r.clone()).unwrap();
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
    let m = Merge::merge(l.clone(), r.clone()).unwrap();
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
    let m = Merge::merge(l.clone(), r.clone()).unwrap();
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
    let m = Merge::merge(l.clone(), r.clone());
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
    let m = Merge::merge(l.clone(), r.clone()).unwrap();
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
    let m = Release::merge(l, r).unwrap();
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
    let m = Merge::merge(l, r);
    dbg!(m);
}

#[test]
fn release_mock() {
    // TODO take the randomness out, I think.
    // But otherwise, this is looking nice.
    let release = create_release();
    assert!(release.artist_credits.len() > 0);
    assert!(release.genres.len() > 0);
    assert!(release.links.len() > 0);
    assert!(release.title == release.release_group.title);

    let l = create_release();
    let r = create_release();
    let m = Release::merge(l, r);
    assert!(m.is_none());

    let l = create_release();
    let r = l.clone();
    let m = Release::merge(l.clone(), r.clone()).unwrap();
    assert!(l == r);
    assert!(l == m);
    assert!(m.genres.len() == l.genres.len());

    let l = Release::default();
    let r = create_release();
    let m = Release::merge(l.clone(), r.clone()).unwrap();
    assert!(l != r);
    assert!(r == m);
    assert!(m.genres.len() == r.genres.len());

    let l = Release {
        title: Some("ff3cf960-8bee-4618-a459-d01158e18a74".to_string()),
        ..Default::default()
    };
    let r = create_release();
    let m = Release::merge(l.clone(), r.clone());
    assert!(m.is_none());
}



