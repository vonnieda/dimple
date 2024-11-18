use std::{collections::HashSet, time::Instant};

use dimple_core::{db::Db, model::{Artist, ArtistCredit, Entity, Genre, KnownIds, Medium, Model, Dimage, Recording, RecordingSource, Release, ReleaseGroup, Track}};

/// - Anything with a key can be merged. Primary or secondary.
/// - Any top level primary object can be merged.
/// - Any secondary object with a related_to that has a key can be merged.
pub fn merge(db: &dyn Db, model: &Model, related_to: &Option<Model>) -> Option<Model> {
    let merged = match model {
        Model::Artist(artist) => db_merge_artist(db, artist),
        Model::Release(release) => db_merge_release(db, release),
        Model::ReleaseGroup(release_group) => db_merge_release_group(db, release_group),
        Model::Recording(recording) => db_merge_recording(db, recording),
        Model::Genre(genre) => db_merge_genre(db, genre),
        Model::Dimage(dimage) => db_merge_dimage(db, dimage, related_to),
        Model::Track(_) => db_merge_model(db, model, related_to),
        _ => panic!("merge({}, {}) not yet implemented", 
            model.entity().type_name(), 
            related_to.clone().map(|related_to| related_to.entity().type_name()).unwrap_or("None".to_string())),
    };
    lazy_link(db, &merged, related_to);
    merged
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

fn db_merge_recording(db: &dyn Db, recording: &Recording) -> Option<Model> {
    let recording: Recording = db_merge_model(db, &recording.model(), &None)?.into();
    db_merge_genres(db, &recording.genres, &recording.model());
    db_merge_artist_credits(db, &recording.artist_credits, &recording.model());
    Some(recording.model())
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
    let track: Track = db_merge_model(db, &track.model(), &Some(medium.model()))?.into();
    db_merge_genres(db, &track.genres, &track.model());
    db_merge_artist_credits(db, &track.artist_credits, &track.model());
    let recording = db_merge_recording(db, &track.recording);
    lazy_link(db, &recording, &Some(track.model()));
    Some(track.model())
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

fn db_merge_dimage(db: &dyn Db, dimage: &Dimage, related_to: &Option<Model>) -> Option<Model> {
    db_merge_model(db, &dimage.model(), related_to)
}

fn db_merge_model(db: &dyn Db, model: &Model, related_to: &Option<Model>) -> Option<Model> {
    let t = Instant::now();
    // TODO does this need to be merging parent as well? Yea, I think it
    // seems obvious we can't merge something to a parent if the parent
    // doesn't exist. 
    // Temporary workaround.
    if let Some(related_to) = related_to {
        if related_to.entity().key().is_none() {
            panic!("db_merge_model called with unmerged related_to");
        }
    }

    // // find a matching model to the specified, merge, save
    // let matching = matching::find_matching_model(db, model, related_to);
    // if let Some(matching) = matching {
    //     let merged = Model::merge(model.clone(), matching);
    //     let inserted = db.insert(&merged).unwrap();
    //     log::debug!("{:04}ms merged {}({})", 
    //         t.elapsed().as_millis(),
    //         inserted.entity().type_name(), 
    //         inserted.entity().key().unwrap());
    //     return Some(inserted)
    // }
    // // if not, insert the new one and link it to the parent
    // else {
    //     if model_valid(model) {
    //         let model = Some(db.insert(model).unwrap());
    //         lazy_link(db, &model, related_to);
    //         {
    //             let model = model.clone().unwrap();
    //             let entity = model.entity();
    //             log::debug!("{:04}ms created {}({})", 
    //                 t.elapsed().as_millis(),
    //                 entity.type_name(), 
    //                 entity.key().unwrap());
    //         }
    //         return model
    //     }
    // }
    // log::warn!("{:04}ms failed {}({})", 
    //     t.elapsed().as_millis(),
    //     model.entity().type_name(), 
    //     model.entity().key().unwrap());
    // None
    todo!()
}

/// Links the two Models if they are both Some. Reduces boilerplate.
fn lazy_link(db: &dyn Db, l: &Option<Model>, r: &Option<Model>) {
    if l.is_some() && r.is_some() {
        db.link(&l.clone().unwrap(), &r.clone().unwrap()).unwrap()
    }
}

/// TODO Goes into the merge trait, maybe. 
fn model_valid(model: &Model) -> bool {
    match model {
        Model::Artist(a) => a.name.is_some() || a.known_ids.musicbrainz_id.is_some(),
        Model::Genre(g) => g.name.is_some(),
        Model::Medium(_m) => true,
        Model::Release(r) => r.title.is_some() || r.known_ids.musicbrainz_id.is_some(),
        Model::ReleaseGroup(rg) => rg.title.is_some(),
        Model::Track(t) => t.title.is_some(),
        Model::Dimage(_p) => true,
        Model::Recording(r) => r.title.is_some() || r.known_ids.musicbrainz_id.is_some(),
        _ => todo!()
    }
}

