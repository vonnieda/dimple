use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Artist, Dimage, Genre, Link, ModelBasics as _, Release};

/// https://musicbrainz.org/doc/Recording
/// https://musicbrainz.org/ws/2/release/4d3ce256-ea71-44c5-8ce9-deb8f1e7dce4?inc=artists+recordings&fmt=json
/// Represents a unique mix or edit. Has title, artist credit, duration, list of ISRCs. Examples (all are different Recordings):
///     Album version of the track "Into the Blue" by "Moby"
///     Remix "Into the Blue (Buzz Boys Main Room Mayhem mix)" by "Moby"
///     Remix "Into the Blue (Underground mix)" by "Moby"
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Recording {
    pub id: Option<String>,
    pub title: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub length_ms: Option<u64>,
    pub lyrics: Option<String>,
    // LRC format (https://en.wikipedia.org/wiki/LRC_(file_format))
    pub synchronized_lyrics: Option<String>,

    pub first_release_date: Option<String>,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::{hash::DefaultHasher};

    use crate::{library::Library, model::Recording};
    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let model = library.save(&Recording::default()).unwrap();
        assert!(model.id.is_some());
    }
}
