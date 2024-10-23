#[derive(Debug, Clone, Default)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub path: String,
    pub liked: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Default)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
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
    #[test]
    fn basics() {
    }
}