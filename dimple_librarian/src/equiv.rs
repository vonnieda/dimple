use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, Track};

/// Equivalent is used to test if two models are considered the same model and
/// can be merged together. Only compares properties. Does not take parent or
/// child objects into consideration.
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

impl Equivalent for Recording {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.key.is_some() && l.key == r.key)
        || (l.isrc.is_some() && l.isrc == r.isrc)
        || KnownIds::equivalent(&l.known_ids, &r.known_ids)
    }
}

impl Equivalent for KnownIds {
    fn equivalent(l: &Self, r: &Self) -> bool {
        (l.musicbrainz_id.is_some() && l.musicbrainz_id == r.musicbrainz_id)
        || (l.discogs_id.is_some() && l.discogs_id == r.discogs_id)
        || (l.lastfm_id.is_some() && l.lastfm_id == r.lastfm_id)
    }
}

#[cfg(test)]
mod test {
    use dimple_core::model::{Genre, KnownIds};

    use crate::equiv::Equivalent;

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

