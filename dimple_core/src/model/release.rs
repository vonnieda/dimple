use dimple_core_macro::ModelSupport;

// https://musicbrainz.org/doc/Release
// https://musicbrainz.org/release/a4864e94-6d75-4ade-bc93-0dabf3521453
// https://musicbrainz.org/ws/2/release/a4864e94-6d75-4ade-bc93-0dabf3521453?fmt=json
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
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

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, Diff}};

    use super::Release;

    #[test]
    fn library_crud() {
        let library = Library::open("file:d13d046d-fb2b-4629-8163-318bf7b47ed6?mode=memory&cache=shared");
        let mut model = library.save(&Release::default());
    }
}