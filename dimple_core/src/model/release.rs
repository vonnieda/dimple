use dimple_core_macro::ModelSupport;

use crate::library::Library;

use super::{Artist, Dimage, Genre, Link, Track};

// https://musicbrainz.org/doc/Release
// https://musicbrainz.org/release/a4864e94-6d75-4ade-bc93-0dabf3521453
// https://musicbrainz.org/ws/2/release/a4864e94-6d75-4ade-bc93-0dabf3521453?fmt=json
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, ModelSupport)]
pub struct Release {
    pub key: Option<String>,
    pub title: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub barcode: Option<String>,
    pub country: Option<String>,
    pub date: Option<String>,
    pub packaging: Option<String>,
    // "Official"
    pub status: Option<String>,
    pub quality: Option<String>,
    pub release_group_type: Option<String>,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,
}

impl Release {
    pub fn artist(&self, library: &Library) -> Option<Artist> {
        self.artists(library).get(0).cloned()
    }

    pub fn artist_name(&self, library: &Library) -> Option<String> {
        self.artist(library).and_then(|a| a.name)
    }

    /// TODO this should return the artists in order, with the primary being
    /// first. I'm not exactly sure how to indicate primary yet.
    pub fn artists(&self, library: &Library) -> Vec<Artist> {
        library.query("
            SELECT a.* FROM ArtistRef ar 
            JOIN Artist a ON (a.key = ar.artist_key) 
            WHERE ar.model_key = ?1
            ORDER BY ar.rowid ASC
        ", (self.key.clone().unwrap(),))
    }

    pub fn genres(&self, library: &Library) -> Vec<Genre> {
        library.query("
            SELECT g.* FROM GenreRef gr 
            JOIN Genre g ON (g.key = gr.genre_key) 
            WHERE gr.model_key = ?1
        ", (self.key.clone().unwrap(),))
    }

    pub fn links(&self, library: &Library) -> Vec<Link> {
        library.query("
            SELECT l.* FROM LinkRef lr 
            JOIN Link l ON (l.key = lr.link_key) 
            WHERE lr.model_key = ?1
        ", (self.key.clone().unwrap(),))
    }

    pub fn tracks(&self, library: &Library) -> Vec<Track> {
        let sql = "
            SELECT Track.* FROM Track
            WHERE Track.release_key = ?1
            ORDER BY media_position ASC, position ASC
        ";
        library.query(sql, (self.key.clone(),))
    }

    pub fn images(&self, library: &Library) -> Vec<Dimage> {
        library.query("
            SELECT d.* FROM DimageRef dr 
            JOIN Dimage d ON (d.key = dr.dimage_key) 
            WHERE dr.model_key = ?1
        ", (self.key.clone().unwrap(),))
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, Diff}};

    use super::Release;

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Release::default());
    }
}