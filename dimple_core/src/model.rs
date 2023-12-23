use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Release {
    pub url: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub art: Vec<Image>,
    pub genres: Vec<Genre>,
    pub tracks: Vec<Track>,
}

impl Release {
    pub fn artist(&self) -> String {
        if let Some(artist) = self.artists.first() {
            return artist.name.clone();
        }
        "".to_string()
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Artist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
    #[serde(default)]
    pub genres: Vec<Genre>,
}

// TODO I think this is gonna need a way to get back to the release. Giving
// more credence to everything just having an ID and the vectors being
// vectors of IDs. And I can add getters that take the request.
// Actually maybe no, because a track might appear in more than one release?
// So I just need to index at a higher level.
// https://musicbrainz.org/doc/MusicBrainz_Database/Schema
// https://wiki.musicbrainz.org/images/a/a7/entity_network_overview.svg makes
// it very clear and more and more this is becoming a MusicBrainz player but
// that's actually really interesting. The entire database isn't that big.
// Maybe that's something else that goes on S3. You download your own copy
// of the database and use it locally plus upload it to S3 so it's yours.
// OKURRR so then I should be focusing on making this an offline MBDB browser?
// And media is just something that gets linked in?
// https://musicbrainz.org/doc/Track

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct Track {
    pub url: String,
    pub title: String,
    pub art: Vec<Image>,
    #[serde(default)]
    pub artists: Vec<Artist>,
    #[serde(default)]
    pub genres: Vec<Genre>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Genre {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Image {
    pub url: String,
}

// TODO Maybe just Artwork, or Art.
pub trait HasArtwork {
    fn art(&self) -> Vec<Image>;
}

impl HasArtwork for Release {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Artist {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Genre {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Playlist {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Track {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

