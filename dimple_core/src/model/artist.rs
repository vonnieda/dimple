use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::known_id::KnownIds;
use super::Genre;

// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub known_ids: KnownIds,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub country: Option<String>,
    
    #[serde(skip)]
    pub genres: Vec<Genre>,
}

impl Hash for Artist {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.name.hash(state);
        self.known_ids.hash(state);
        self.disambiguation.hash(state);
        self.summary.hash(state);
        // self.links.hash(state);
        self.country.hash(state);
        self.genres.hash(state);
    }
}

impl Display for Artist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}{}", 
            self.name.clone().unwrap_or_default(),
            self.disambiguation.clone().map(|d| format!(" ({})", d)).unwrap_or_default()
        ).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert!(Artist::default().model().entity().type_name() == "Artist")
    }

    #[test]
    fn display() {
        let artist = Artist {
            name: Some("Blue Plate".to_string()),
            disambiguation: Some("Small one".to_string()),
            ..Default::default()
        };
        assert!(artist.to_string() == "Blue Plate (Small one)");
        let artist = Artist {
            name: Some("Red Plate".to_string()),
            ..Default::default()
        };
        assert!(artist.to_string() == "Red Plate");
    }
}