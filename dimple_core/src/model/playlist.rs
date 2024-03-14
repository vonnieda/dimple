


use serde::Deserialize;
use serde::Serialize;

use crate::collection::Collection;


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Playlist {
    pub name: String,
}


