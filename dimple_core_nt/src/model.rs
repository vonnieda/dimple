#[derive(Debug, Clone)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub path: Option<String>,
}

