use dimple_core::model::{Artist, ArtistCredit, Dimage, Genre, KnownIds, Medium, Model, Recording, Release, ReleaseGroup, Track};

/// Equivalent is used to test if two models are considered the same model and
/// can be merged together. Only compares properties. Does not take parent or
/// child objects into consideration.
/// 
/// TODO Ultimately, I think Merge, Equivalent, and maybe a another merge
/// into Merge and may need to be able to indicate when merging one into
/// another would be a conflict. 
/// 
/// I think equivalent is still wrong. I think it's mergable() -> bool.

pub trait Equivalent {
    fn equivalent(l: &Self, r: &Self) -> bool;
}

impl Equivalent for Artist {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.name.is_some() && l.name == r.name && l.disambiguation == r.disambiguation)
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for ReleaseGroup {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        // TODO needs artist credits
        // || (l.title.is_some() && l.title == r.title && Recording::equivalent(&l.recording, &r.recording))
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for Release {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        // TODO needs artist credits
        // || (l.title.is_some() && l.title == r.title && Recording::equivalent(&l.recording, &r.recording))
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for Recording {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.isrc.is_some() && l.isrc == r.isrc)
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for Medium {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.position.is_some() && l.position == r.position)
    }
}

impl Equivalent for Track {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.title.is_some() && l.title == r.title && Recording::equivalent(&l.recording, &r.recording))
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for Genre {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.name.is_some() && l.name == r.name && l.disambiguation == r.disambiguation)
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for ArtistCredit {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.name.is_some() && l.name == r.name && Artist::equivalent(&l.artist, &r.artist))
    }
}

impl Equivalent for KnownIds {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.musicbrainz_id.is_some() && l.musicbrainz_id == r.musicbrainz_id)
        || (l.discogs_id.is_some() && l.discogs_id == r.discogs_id)
        || (l.lastfm_id.is_some() && l.lastfm_id == r.lastfm_id)
    }
}

impl Equivalent for Dimage {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.data.len() == r.data.len() && l.width == r.width && l.height == r.height)
    }
}

impl Equivalent for Model {
    fn equivalent(l: &Self, r: &Self) -> bool {
        match ((l, r)) {
            (Model::Artist(l), Model::Artist(r)) => Artist::equivalent(l, r),
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => ReleaseGroup::equivalent(l, r),
            (Model::Release(l), Model::Release(r)) => Release::equivalent(l, r),
            (Model::Track(l), Model::Track(r)) => Track::equivalent(l, r),
            (Model::Medium(l), Model::Medium(r)) => Medium::equivalent(l, r),
            (Model::Recording(l), Model::Recording(r)) => Recording::equivalent(l, r),
            (Model::Genre(l), Model::Genre(r)) => Genre::equivalent(l, r),
            (Model::ArtistCredit(l), Model::ArtistCredit(r)) => ArtistCredit::equivalent(l, r),
            (Model::Dimage(l), Model::Dimage(r)) => Dimage::equivalent(l, r),
            _ => todo!()
        }
    }
}

impl <T: Equivalent + Clone> Equivalent for Vec<T> {
    fn equivalent(l: &Self, r: &Self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use dimple_core::model::{Genre, KnownIds};

    use crate::equivalent::Equivalent;

    #[test]
    fn genre() {
        let l = Genre {
            name: Some("fishjazz".to_string()),
            summary: Some("Jazz, by fishes.".to_string()),
            ..Default::default()
        };
        let r = Genre {
            name: Some("fishjazz".to_string()),
            known_ids: KnownIds { 
                musicbrainz_id: Some("9999-9999-9999-9999".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(Genre::equivalent(&l, &r));
    }
}

