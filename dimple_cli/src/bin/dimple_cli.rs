use anyhow::Result;
use dimple_core::{db::{Db, MemoryDb, SqliteDb}, model::{Artist, Model, Release}};

fn main() -> Result<()> {
    let db = SqliteDb::new("test.db");

    let artist1: Artist = db.insert(&Artist {
        name: Some("Rick and Morty".to_string()),
        ..Default::default()
    }.into())?.into();

    let artist2: Artist = db.insert(&Artist {
        name: Some("Infected Mushroom".to_string()),
        ..Default::default()
    }.into())?.into();

    let artist3: Artist = db.insert(&Artist {
        name: Some("Hoodie Poo".to_string()),
        ..Default::default()
    }.into())?.into();

    let release1: Release = db.insert(&Release {
        title: Some("Mega Seeds".to_string()),
        ..Default::default()
    }.into())?.into();

    let release2: Release = db.insert(&Release {
        title: Some("Boss La Rosh".to_string()),
        ..Default::default()
    }.into())?.into();

    let release3: Release = db.insert(&Release {
        title: Some("All Together Now".to_string()),
        ..Default::default()
    }.into())?.into();

    db.link(&release1.clone().into(), &artist1.clone().into())?;
    db.link(&release2.clone().into(), &artist2.clone().into())?;
    db.link(&release3.clone().into(), &artist1.clone().into())?;
    db.link(&release3.clone().into(), &artist2.clone().into())?;
    db.link(&release3.clone().into(), &artist3.clone().into())?;

    let artists: Vec<Artist> = db
        .list(&Model::Artist(Artist::default()), None)?
        .map(Into::into)
        .collect();
    println!("{:?}", artists);

    let releases: Vec<Release> = db
        .list(&Model::Release(Release::default()), None)?
        .map(Into::into)
        .collect();
    println!("{:?}", releases);

    let artist1_releases: Vec<Release> = db
        .list(
            &Model::Release(Release::default()),
            Some(&Model::Artist(artist1)),
        )?
        .map(Into::into)
        .collect();
    println!("{:?}", artist1_releases);

    let artist2_releases: Vec<Release> = db
        .list(
            &Model::Release(Release::default()),
            Some(&Model::Artist(artist2)),
        )?
        .map(Into::into)
        .collect();
    println!("{:?}", artist2_releases);

    let release2_artists: Vec<Artist> = db
        .list(
            &Model::Artist(Artist::default()),
            Some(&Model::Release(release2)),
        )?
        .map(Into::into)
        .collect();
    println!("{:?}", release2_artists);

    let artists: Vec<Artist> = db.list(&Model::Artist(Artist::default()), None)?.map(Into::into).collect();
    for artist in artists {
        println!("{}", artist.name.clone().unwrap());
        let releases: Vec<Release> = db.list(&Model::Release(Release::default()), Some(&Model::Artist(artist)))?.map(Into::into).collect();
        for release in releases {
            println!("  {}", release.title.unwrap());
        }
    }

    Ok(())
}
