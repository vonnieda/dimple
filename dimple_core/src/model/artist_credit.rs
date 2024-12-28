use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct ArtistCredit {
    pub key: Option<String>,
    pub model_key: String,
    pub artist_key: String,
}

// Old reference
// // https://musicbrainz.org/doc/Artist_Credits
// // > Artist credits can be added to tracks, recordings, releases, and release groups. 
// // Note that this combines portions of the artist_credit_name table, too.
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ModelSupport)]
// pub struct ArtistCredit {
//     pub key: Option<String>,
//     pub name: Option<String>,
//     pub join_phrase: Option<String>,
    
//     #[serde(skip)]
//     pub artist: Artist,
// }

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, ArtistCredit, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:29832924-08a3-4000-b6d9-73f10920d387?mode=memory&cache=shared");
        let artist = library.save(&Artist::default());
        let track = library.save(&Track::default());
        let model = ArtistCredit {
            key: None,
            artist_key: artist.key.unwrap(),
            model_key: track.key.unwrap(),
        };
        let mut model = library.save(&model);
        dbg!(model);
        // assert!(model.key.is_some());
        // assert!(model.model_key.is_none());
        // model.model_key = Some("Name".to_string());
        // let model = library.save(&model);
        // let model: Artist = library.get(&model.key.unwrap()).unwrap();
        // assert!(model.model_key == Some("Name".to_string()));
    }
}

