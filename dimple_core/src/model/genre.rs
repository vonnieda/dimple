use dimple_core_macro::ModelSupport;

// https://musicbrainz.org/doc/Genre
#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Genre {
    pub key: Option<String>,
    pub name: Option<String>,
    // pub known_ids: KnownIds,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    // pub links: HashSet<String>,
}

// impl Hash for Genre {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.key.hash(state);
//         self.name.hash(state);
//         self.known_ids.hash(state);
//         self.disambiguation.hash(state);
//         self.summary.hash(state);
//         // self.links.hash(state);
//     }
// }