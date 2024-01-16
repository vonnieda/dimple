use serde::Deserialize;
use serde::Serialize;

// TODO feels more like attributed things are their own objects and not fields
// on structs that may also be attributed.

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleArtist {
    pub id: String,
    pub name: String,
    pub disambiguation: String,
    pub summary: Option<Attributed<String>>,
    pub release_groups: Option<Vec<DimpleReleaseGroup>>,
    pub releases: Option<Vec<DimpleRelease>>,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
    #[serde(default)]
    pub fetched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DimpleReleaseGroup {
    pub id: String,
    pub title: String,
    pub disambiguation: String,
    pub summary: Option<Attributed<String>>,
    pub primary_type: String,
    pub first_release_date: String,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
    pub releases: Option<Vec<DimpleRelease>>,
    pub artists: Option<Vec<DimpleArtist>>,
    #[serde(default)]
    pub fetched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DimpleRelease {
    pub id: String,
    pub title: String,
    pub disambiguation: String,
    pub summary: Option<Attributed<String>>,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
    pub artists: Option<Vec<DimpleArtist>>,
    pub status: String,
    pub date: String,
    pub packaging: String,
    pub country: String,
    pub barcode: String,
    pub asin: String,
    pub release_group: Option<DimpleReleaseGroup>,
    #[serde(default)]
    pub fetched: bool,
    pub media: Vec<DimpleMedia>,
}

// ReleseGroup -> Release -> Media -> Track -> Recording

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleMedia {
    pub title: Option<String>,
    pub position: Option<u32>,
    pub track_count: u32,
    pub disc_count: Option<u32>,
    pub format_id: Option<String>,
    pub format: Option<String>,
    pub tracks: Option<Vec<DimpleTrack>>,
    #[serde(default)]
    pub fetched: bool,
}

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleTrack {
    pub recording: DimpleRecording,
    pub title: String,
    pub number: String,
    pub length: Option<u32>,
    pub position: u32,
    pub id: String,
    #[serde(default)]
    pub fetched: bool,
}

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleRecording {
    /// See [MusicBrainz Identifier](https://musicbrainz.org/doc/MusicBrainz_Identifier).
    pub id: String,
    /// The title of the recording.
    pub title: String,

    pub video: Option<bool>,
    /// The length of the recording. It's only entered manually for
    /// [standalone recordings](https://musicbrainz.org/doc/Standalone_Recording). For recordings
    /// that are being used on releases, the recording length is the median length of all tracks
    /// (that have a track length) associated with that recording. If there is an even number of
    /// track lengths, the smaller median candidate is used.
    pub length: Option<u32>, // TODO: CUSTOM Deserialized to make this a duration
    /// The disambiguation comments are fields in the database used to help distinguish identically
    /// named artists, labels and other entities.
    pub disambiguation: Option<String>,
    /// The International Standard Recording Code assigned to the recording.
    pub isrcs: Option<Vec<String>>,
    // pub relations: Option<Vec<Relation>>,
    // pub releases: Option<Vec<Release>>,
    /// Artist credits indicate who is the main credited artist (or artists) for releases, release
    /// groups, tracks and recordings, and how they are credited.
    // pub artist_credit: Option<Vec<ArtistCredit>>,
    /// Aliases are alternate names for a recording.
    // pub aliases: Option<Vec<Alias>>,
    // pub tags: Option<Vec<Tag>>,
    // pub rating: Option<Rating>,
    /// Genres are currently supported in MusicBrainz as part of the tag system.
    // pub genres: Option<Vec<Genre>>,
    /// Annotations are text fields, functioning like a miniature wiki, that can be added to any
    /// existing artists, labels, recordings, releases, release groups and works.
    pub annotation: Option<String>,
    #[serde(default)]
    pub fetched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleGenre {
    // pub id: String,
    pub name: String,
    pub count: u32,
    pub description: Option<Attributed<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DimpleRelation {
    pub content: DimpleRelationContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DimpleRelationContent {
    Url(DimpleUrl),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleUrl {
    pub id: String,
    pub resource: String,
}

/// An attribution for a piece of data. Indicates where the data was sourced,
/// who owns it, and under what license it is being used.
/// DimpleAttribution {
///     text: "Wikipedia content provided under the terms of the Creative Commons BY-SA license",
///     url: "https://en.wikipedia.org/wiki/Brutus_%28Belgian_band%29"
///     license: "CC-BY-SA"
///     copyright_holder: "WikiCommons"
/// }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Attributed<T> {
    pub value: T,

    pub text: String,
    pub url: String,
    pub license: String,
    pub copyright_holder: String,
}