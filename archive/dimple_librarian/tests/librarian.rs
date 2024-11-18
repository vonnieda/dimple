use std::collections::HashSet;

use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, Release, ReleaseGroup, Track};
use dimple_librarian::librarian::Librarian;

#[test]
fn merge_basics() {
    let lib = Librarian::new("test.db");
    lib.merge2(Artist {
        name: Some("Artist 1".to_string()),
        genres: vec![
            Genre {
                name: Some("rock".to_string()),
                ..Default::default()
            },
            Genre {
                name: Some("funk".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    assert!(lib.list2::<_, Artist>(Artist::default(), None).unwrap().count() == 1);
    // assert!(lib.list2::<_, Genre>(Genre::default(), None).unwrap().count() == 2);
}

#[test]
fn basics() {
    let lib = Librarian::new("test.db");
    lib.merge2(Artist {
        name: Some("a".to_string()),
        ..Default::default()
    });
    lib.merge2(Artist {
        name: Some("a".to_string()),
        summary: Some("s".to_string()),
        ..Default::default()
    });
    lib.merge2(Artist {
        name: Some("a".to_string()),
        disambiguation: Some("a1".to_string()),
        summary: Some("s".to_string()),
        ..Default::default()
    });
    lib.merge2(Artist {
        key: Some("96b47db5-f2ed-4f60-9a9d-a3b91d461847".to_string()),
        name: Some("a".to_string()),
        disambiguation: Some("a1".to_string()),
        summary: Some("s".to_string()),
        ..Default::default()
    });
    let artists = lib.list2(Artist::default(), None::<Artist>).unwrap().collect::<Vec<_>>();
    assert!(artists.len() == 3);
}

#[test]
fn merge_system() {
    let lib = Librarian::new("test.db");
    lib.merge2(Artist {
        name: Some("Jason von Nieda".to_string()),
        genres: vec![
            Genre {
                name: Some("metal".to_string()),
                ..Default::default()
            }
        ],
        ..Default::default()
    });
    lib.merge2(Artist {
        name: Some("Jason von Nieda".to_string()),
        genres: vec![
            Genre {
                name: Some("metal".to_string()),
                ..Default::default()
            },
            Genre {
                name: Some("folk metal".to_string()),
                ..Default::default()
            }
        ],
        ..Default::default()
    });
    lib.merge2(Artist {
        name: Some("Jason vonNieda".to_string()),
        genres: vec![
            Genre {
                name: Some("pop".to_string()),
                ..Default::default()
            },
            Genre {
                name: Some("rock".to_string()),
                ..Default::default()
            }
        ],
        ..Default::default()
    });
    lib.merge2(Release {
        title: Some("Funky Metal".to_string()),
        ..Default::default()
    });
    let artists = lib.list2(Artist::default(), None::<Artist>).unwrap().collect::<Vec<_>>();
    assert!(artists.len() == 1);
    assert!(artists[0].genres.len() == 2);
    let genres = lib.list2(Genre::default(), None::<Genre>).unwrap().collect::<Vec<_>>();
    assert!(genres.len() == 2);
}

