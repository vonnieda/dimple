use serde::Deserialize;
use serde::Serialize;
use ulid::Ulid;

use crate::model::Genre;
use crate::model::Image;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,    
    pub mbid: Option<String>,

    pub name: String,
    #[serde(default)]
    pub art: Vec<Image>,
    #[serde(default)]
    pub genres: Vec<Genre>,
}

impl Default for Artist {
    fn default() -> Self {
        Artist {
            id: Ulid::new().to_string(),
            mbid: None,

            name: "".to_string(),
            art: vec![],
            genres: vec![],
        }
    }
}

impl Eq for Artist {}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Artist {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

