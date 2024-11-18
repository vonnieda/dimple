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

    pub barcode: Option<String>,
    pub country: Option<String>,
    pub date: Option<String>,
    pub packaging: Option<String>,
    pub status: Option<String>,
    pub quality: Option<String>,

    pub release_group_key: Option<String>,

    // pub known_ids: KnownIds,
    // pub links: HashSet<String>,
    // pub artist_credits: Vec<ArtistCredit>,
    // pub genres: Vec<Genre>,
    // pub media: Vec<Medium>,
}

