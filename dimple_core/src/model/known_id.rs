use serde::Serialize;
use serde::Deserialize;

// I think this becomes a struct 
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnownId {
    MusicBrainzId(String),
    DiscogsId(String),
    LastFmId(String),
    WikidataId(String),
    SpotifyId,
    DeezerId,
    TidalId,
    YouTubeId,
    ItunesStoreId,
    AppleMusicId, // TODO same as above?
    QobuzId,
    BandcampUrl,
    SoundCloud,

    // https://musicbrainz.org/doc/Barcode
    Barcode(String),

    // https://musicbrainz.org/doc/ISRC
    ISRC(String),

    // https://musicbrainz.org/doc/ASIN
    ASIN(String),

    AcoustId(String),
    AcoustIdFingerprint(String),
}

