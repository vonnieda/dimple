use image::DynamicImage;

pub mod navidrome;
pub mod local;
pub mod image_cache;

pub trait MusicLibrary {
    /// All of the releases in the library. This function may block for
    /// a long time while it retrieves information. The caller should
    /// cache the information.
    fn releases(self: &Self) -> Vec<Release>;

    /// Add or update a release into the library. Returns the merged release.
    /// If data in the existing release and the new one differ the new data
    /// are preferred.
    fn merge_release(self: &Self, _release: &Release) -> Result<Release, String> {
        Err("not implemented".to_string())
    }
}

#[derive(Default, Clone)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub cover_art: Option<DynamicImage>,
    pub genre: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub cover_art: Option<DynamicImage>,
    // releases: Vec<Release>,
}

#[derive(Default, Clone)]
pub struct Track {
    pub title: String,
    pub stream: Vec<u8>,
    pub artists: Vec<Artist>,
}

#[derive(Default, Clone)]
pub struct Genre {
    pub name: String,
    pub cover_art: Option<DynamicImage>,
}



#[derive(Default)]
pub struct EmptyMusicLibrary {

}

impl MusicLibrary for EmptyMusicLibrary  {
    fn releases(self: &Self) -> Vec<Release> {
        Vec::new()
    }
}


// #[derive(Default, Clone)]
// pub struct Release {
//     pub id: String,
//     pub title: String,
//     pub artists: Vec<String>,
//     pub cover_art: Option<DynamicImage>,
//     pub genres: Vec<String>,
//     pub tracks: Vec<Track>,
// }

// #[derive(Default, Clone)]
// struct Artist {
//     id: String,
//     name: String,
//     releases: Vec<Release>,
// }

// #[derive(Default, Clone)]
// struct Track {
//     title: String,
//     stream: Vec<u8>,
//     artists: Vec<Artist>,
// }

// #[derive(Default, Clone)]
// struct Genre {
//     name: String,
// }

// #[derive(Default)]
// pub struct EmptyMusicLibrary {

// }
