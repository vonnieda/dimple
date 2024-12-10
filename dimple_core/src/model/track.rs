use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub liked: bool,
    pub plays: u32,
    // TODO Duration, I think, and probably ns vs ms.
    pub length_ms: Option<u64>,
    // pub media_position: Option<u32>,

    // pub save: bool,
    // pub download: bool,
    // pub disambiguation: Option<String>,
    // pub summary: Option<String>,

    // pub musicbrainz_id: Option<String>,
    // pub discogs_id: Option<String>,
    // pub lastfm_id: Option<String>,
    // pub wikidata_id: Option<String>,
    // pub spotify_id: Option<String>,

    pub lyrics: Option<String>,
}

// // https://musicbrainz.org/doc/Track
// // https://musicbrainz.org/ws/2/release/4d3ce256-ea71-44c5-8ce9-deb8f1e7dce4?inc=aliases%2Bartist-credits%2Blabels%2Bdiscids%2Brecordings&fmt=json
// // > This entity is not visible to users on its own, only in the context of a
// // release. It has an MBID, and contains a link to a recording, a title, 
// // artist credit and position on its associated medium. 
// // In the schema image it also has a medium (ref)
// #[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
// pub struct Track {
//     pub key: Option<String>,
//     pub title: Option<String>,
//     // pub known_ids: KnownIds,

//     // pub length: Option<u32>,
//     // A text description of the position in the media, such as A1
//     // pub number: Option<u32>,
//     // 1 based ordinal within the media
//     // pub position: Option<u32>,

//     pub recording_key: String,

//     // #[serde(skip)]
//     // pub recording: Recording,
//     // #[serde(skip)]
//     // pub genres: Vec<Genre>,
//     // #[serde(skip)]
//     // pub artist_credits: Vec<ArtistCredit>,
// }

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:0557f771-4697-4d8d-807b-9576381b50b4?mode=memory&cache=shared");
        let mut model = library.save(&Track::default());
        assert!(model.key.is_some());
        assert!(model.artist.is_none());
        model.artist = Some("Artist".to_string());
        let model = library.save(&model);
        let model: Track = library.get(&model.key.unwrap()).unwrap();
        assert!(model.artist == Some("Artist".to_string()));
    }

    #[test]
    fn diff() {
        let a = Track::default();
        let b = Track {
            key: Some("key".to_string()),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
            liked: true,
            ..Default::default()
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 5);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("artist".to_string()));
        assert!(diff[2].field == Some("album".to_string()));
        assert!(diff[3].field == Some("title".to_string()));
        assert!(diff[4].field == Some("liked".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Track::default();
        let b = Track {
            key: Some("key".to_string()),
            artist: Some("artist".to_string()),
            album: Some("album".to_string()),
            title: Some("title".to_string()),
            liked: true,
            ..Default::default()
        };
        let diff = a.diff(&b);
        let mut c = Track::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }
}