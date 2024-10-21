#[derive(Debug, Clone, Default)]
pub struct Track {
    pub key: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub tracks: Vec<Track>,
}

pub struct Artist {
}

#[cfg(test)]
mod tests {
    #[test]
    fn basics() {
    }
}