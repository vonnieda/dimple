use dimple_core_macro::ModelSupport;

// #[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
// pub struct Artist {
//     pub key: Option<String>,
//     pub name: Option<String>,
// }

// https://musicbrainz.org/doc/Artist
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,

    pub country: Option<String>,
    
    pub saved: bool,

    // pub known_ids: KnownIds,
    // pub links: HashSet<String>,
    // pub genres: Vec<Genre>,
}
