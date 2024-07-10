use dimple_core::{
    db::Db,
    model::{Artist, ArtistCredit, Entity, Genre, Medium, Model, Track},
};
use dimple_librarian::sqlite_db::SqliteDb;

#[test]
fn basics() {
    let db = SqliteDb::new(":memory:").unwrap();
    let artist = db
        .set(&Artist {
            name: Some("say hi".to_string()),
            ..Default::default()
        })
        .unwrap();
    let artist2: Artist = db.get(&artist.key.clone().unwrap()).unwrap().unwrap();
    assert!(artist == artist2);
}

#[test]
fn get() {
    let db = SqliteDb::new(":memory:").unwrap();
    let artist = db
        .set(&Artist {
            key: Some("b5965896-9124-41c1-adfc-ea924df70d05".to_string()),
            name: Some("say hi".to_string()),
            ..Default::default()
        })
        .unwrap();

    let artist2: Artist = db
        .get("b5965896-9124-41c1-adfc-ea924df70d05")
        .unwrap()
        .unwrap();
    assert!(artist == artist2);

    let artist3: Option<Artist> = db.get("").unwrap();
    assert!(artist3.is_none());

    let artist4: Option<Artist> = db.get("2d6f8f82-f57d-4f83-ab5f-f13c1471bc17").unwrap();
    assert!(artist4.is_none());
}

#[test]
fn query() {
    let db = SqliteDb::new(":memory:").unwrap();

    db.set(&Artist {
        name: Some("say hi".to_string()),
        ..Default::default()
    })
    .unwrap();
    db.set(&Artist {
        name: Some("say hello".to_string()),
        ..Default::default()
    })
    .unwrap();
    db.set(&Artist {
        name: Some("say howdy".to_string()),
        ..Default::default()
    })
    .unwrap();
    db.set(&Artist {
        name: Some("say yo".to_string()),
        ..Default::default()
    })
    .unwrap();

    let artists: Vec<Artist> = db.query("SELECT doc FROM Artist").unwrap().collect();
    assert!(artists.len() == 4);
    let artists: Vec<Artist> = db
        .query("SELECT doc FROM Artist WHERE doc->>'name' LIKE 'say h%'")
        .unwrap()
        .collect();
    assert!(artists.len() == 3);
}

#[test]
fn transactions() {
    let db = SqliteDb::new(":memory:").unwrap();

    db.begin().unwrap();
    db.set(&Artist {
        key: Some("3cbd37cd-e019-430e-90eb-6ef35a4e1b70".to_string()),
        name: Some("say hi".to_string()),
        ..Default::default()
    })
    .unwrap();
    db.rollback().unwrap();
    let artist: Option<Artist> = db.get("3cbd37cd-e019-430e-90eb-6ef35a4e1b70").unwrap();
    assert!(artist.is_none());

    db.begin().unwrap();
    db.set(&Artist {
        key: Some("3cbd37cd-e019-430e-90eb-6ef35a4e1b70".to_string()),
        name: Some("say howdy".to_string()),
        ..Default::default()
    })
    .unwrap();
    db.commit().unwrap();
    let artist: Artist = db
        .get("3cbd37cd-e019-430e-90eb-6ef35a4e1b70")
        .unwrap()
        .unwrap();
    assert!(artist.name == Some("say howdy".to_string()));
}

#[test]
fn children() {
    let db = SqliteDb::new(":memory:").unwrap();
    let artist = Artist {
        name: Some("Artist 1".to_string()),
        genres: vec![
            Genre {
                name: Some("Genre 1".to_string()),
                ..Default::default()
            },
            Genre {
                name: Some("Genre 2".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert!(artist.genres.len() == 2);
    let artist = db.set(&artist).unwrap();
    assert!(artist.genres.len() == 0);
    let artist: Artist = db.get(&artist.key.unwrap()).unwrap().unwrap();
    assert!(artist.genres.len() == 0);
}

// #[test]
// fn relations() {
//     let db = SqliteDb::new(":memory:").unwrap();

//     db.set(&Genre {
//         key: Some("fa8923db-836f-43ce-92f8-1fcd6eca4adc".to_string()),
//         name: Some("Genre 1".to_string()),
//         ..Default::default()
//     })
//     .unwrap();
//     db.set(&Genre {
//         key: Some("f199bb03-0f37-486e-9b6a-74b026cb17ff".to_string()),
//         name: Some("Genre 2".to_string()),
//         ..Default::default()
//     })
//     .unwrap();
//     db.set(&Genre {
//         key: Some("c74eb140-a673-4890-abbb-e20f7cb66d63".to_string()),
//         name: Some("Genre 3".to_string()),
//         ..Default::default()
//     })
//     .unwrap();
//     db.set(&Genre {
//         key: Some("434f7881-8f88-420f-9320-f299f389e6eb".to_string()),
//         name: Some("Genre 4".to_string()),
//         ..Default::default()
//     })
//     .unwrap();

//     db.set(&Artist {
//         key: Some("fae7a3f6-812e-4372-a8a6-6781e12afa66".to_string()),
//         name: Some("Artist 1".to_string()),
//         genres: vec![
//             Genre {
//                 key: Some("fa8923db-836f-43ce-92f8-1fcd6eca4adc".to_string()),
//                 ..Default::default()
//             },
//             Genre {
//                 key: Some("c74eb140-a673-4890-abbb-e20f7cb66d63".to_string()),
//                 ..Default::default()
//             },
//         ],
//         ..Default::default()
//     })
//     .unwrap();

//     let genres: Vec<Genre> = db
//         .query(
//             //     "SELECT doc
//             // FROM Genre AS g
//             // WHERE g.key IN (
//             //     SELECT json_extract(je.value, '$.key')
//             //     FROM Artist AS a, json_each(a.doc, '$.genres') AS je
//             //     WHERE a.key = 'fae7a3f6-812e-4372-a8a6-6781e12afa66')",
//             "SELECT g.doc
//             FROM Genre AS g
//             INNER JOIN (
//                 SELECT json_extract(je.value, '$.key') AS genre_key
//                 FROM Artist AS a, json_each(a.doc, '$.genres') AS je
//                 WHERE a.key = 'fae7a3f6-812e-4372-a8a6-6781e12afa66'
//             ) AS extracted_genres
//             ON g.key = extracted_genres.genre_key;",
//         )
//         .unwrap()
//         .collect();
//     assert!(genres.len() == 2);

//     let genres: Vec<Genre> = db.query("SELECT doc FROM Genre").unwrap().collect();
//     assert!(genres.len() == 4);
// }

#[test]
fn multiple_connections() {
    // This syntax creates an in-memory database that multiple connections
    // can attach to, the same as if it was an actual file.
    // https://www.sqlite.org/inmemorydb.html
    let path = "file:memdb1?mode=memory&cache=shared";
    let db_1 = SqliteDb::new(path).unwrap();
    let db_2 = SqliteDb::new(path).unwrap();
    let db_3 = SqliteDb::new(path).unwrap();
    db_1.set(&Artist::default()).unwrap();
    db_2.set(&Artist::default()).unwrap();
    assert!(
        db_3.query::<Artist>("SELECT doc FROM Artist")
            .unwrap()
            .count()
            == 2
    );
}

#[test]
fn secondary_entity() {
    let db = SqliteDb::new(":memory:").unwrap();
    let model = db.set(&ArtistCredit::default()).unwrap();
    assert!(model.key.is_some());
    let model = db.set(&Medium::default()).unwrap();
    assert!(model.key.is_some());
    let model = db.set(&Track::default()).unwrap();
    assert!(model.key.is_some());
}

#[test]
fn db_trait() {
    let db: &dyn Db = &SqliteDb::new(":memory:").unwrap();
    let artist = db.insert(&Artist::default().model()).unwrap();
    let _ = db.insert(&Genre::default().model()).unwrap();
    let _ = db.insert(&Genre::default().model()).unwrap();
    let genre = db.insert(&Genre::default().model()).unwrap();
    db.link(&genre, &artist).unwrap();
    let artists: Vec<Model> = db
        .list(&Artist::default().model(), &None)
        .unwrap()
        .collect();
    let genres: Vec<Model> = db.list(&Genre::default().model(), &None).unwrap().collect();
    let artist_genres: Vec<Model> = db.list(&Genre::default().model(), &Some(artist.clone())).unwrap().collect();
    assert!(artists.len() == 1);
    assert!(genres.len() == 3);
    assert!(artist_genres.len() == 1);
    let artist2: Artist = db.get(&artist).unwrap().unwrap().into();
    assert!(artist.entity().key() == artist2.key);
    assert!(artist2.name.is_none());
    let _ = db.insert(&Artist {
        name: Some("cat".to_string()),
        ..Default::default()
    }.model()).unwrap();
    let _ = db.insert(&Artist {
        name: Some("calf".to_string()),
        ..Default::default()
    }.model()).unwrap();
    let _ = db.insert(&Artist {
        name: Some("dog".to_string()),
        ..Default::default()
    }.model()).unwrap();
    let artists: Vec<_> = db.query("SELECT doc FROM Artist WHERE doc->>'Artist.name' LIKE 'ca%'").unwrap().collect();
    assert!(artists.len() == 2);
}
