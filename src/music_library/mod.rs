use image::DynamicImage;

pub mod navidrome;
pub mod example;

pub trait MusicLibrary {
    fn releases(self: &Self) -> Vec<Release>;
}

// TODO becomes a Trait so that libraries can return their own implementation
// that can call back for e.g. stream, save, whatever.
#[derive(Default, Clone)]
pub struct Release {
    pub title: String,
    pub artist: Option<String>,
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