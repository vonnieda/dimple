use image::DynamicImage;

pub mod navidrome;
pub mod example;

pub trait MusicLibrary {
    fn releases(self: &Self) -> Vec<Release>;
}

pub struct Release {
    pub title: String,
    pub artist: Option<String>,
    pub release_year: Option<u32>,
    pub cover_image: Option<DynamicImage>,
}

#[derive(Default)]
pub struct EmptyMusicLibrary {

}

impl MusicLibrary for EmptyMusicLibrary  {
    fn releases(self: &Self) -> Vec<Release> {
        return Vec::new();
    }
}