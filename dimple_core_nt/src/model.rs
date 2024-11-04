#[derive(Debug, Clone, Default, PartialEq)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub path: String,
    pub liked: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChangeLog {
    pub actor: String,
    pub timestamp: String,
    pub model: String,
    pub key: String,
    pub op: String,
    pub field: Option<String>,
    pub value: Option<String>,
}

#[cfg(test)]
mod tests {
    // use super::Track;

    #[test]
    fn it_works() {
        // let old = Track {
        //     key: Some("2b787c14-85df-462b-b81a-ff3ded9f5f7c".to_string()),
        //     artist: Some("The Funky Bunch".to_string()),
        //     ..Default::default()
        // };
        // let new = Track {
        //     key: Some("2b787c14-85df-462b-b81a-ff3ded9f5f7c".to_string()),
        //     artist: Some("The Wild Bunch".to_string()),
        //     album: Some("Walkin' Around".to_string()),
        //     ..Default::default()
        // };

        // let diff = old.diff(&new);

        // dbg!(diff);
    }
}