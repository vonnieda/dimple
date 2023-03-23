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

// TODO becomes a Trait so that libraries can return their own implementation
// that can call back for e.g. stream, save, whatever.
#[derive(Default, Clone)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub cover_image: Option<DynamicImage>,
    pub genre: Option<String>,
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
