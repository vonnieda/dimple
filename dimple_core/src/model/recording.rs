use serde::{Deserialize, Serialize};

use crate::library::Library;

use super::{Artist, Dimage, Genre, Link, ModelBasics as _, Release};

// https://musicbrainz.org/doc/Recording
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
    // pub instrumental: Option<bool>;
    // LRC format (https://en.wikipedia.org/wiki/LRC_(file_format))
    pub synchronized_lyrics: Option<String>,

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
        let mut model = library.save(&Recording::default());
    }
}
