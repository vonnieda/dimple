use std::collections::HashSet;

use dimple_core::model::{Artist, ArtistCredit, Genre, KnownIds, Medium, Recording, Release, ReleaseGroup, Track};

pub fn create_genres(count: usize) -> Vec<Genre> {
    let mut genres = vec![];
    for _ in 0..count {
        genres.push(Genre {
            name: Some(GENRE_OPTIONS.get(fakeit::misc::random(0, GENRE_OPTIONS.len() - 1)).unwrap().to_string()),
            ..Default::default()
        });
    }
    genres
}

pub fn create_links(count: usize) -> HashSet<String> {
    let mut links = HashSet::new();
    for _ in 0..count {
        links.insert(format!("https://{}/{}/{}", 
            fakeit::internet::domain_name(), 
            fakeit::words::word(),
            fakeit::unique::uuid_v4()));
    }
    links
}

pub fn create_known_ids() -> KnownIds {
    KnownIds {
        musicbrainz_id: Some(fakeit::unique::uuid_v4()),
        ..Default::default()
    }
}

pub fn create_artist() -> Artist {
    Artist {
        name: Some(fakeit::name::full()),
        summary: Some(fakeit::hipster::paragraph(1, 4, 40, " ".to_string())),
        country: Some(fakeit::address::country_abr()),
        disambiguation: Some(fakeit::address::country()),
        links: create_links(fakeit::misc::random(0, 5)),
        known_ids: create_known_ids(),
        genres: create_genres(fakeit::misc::random(1, 5)),
        ..Default::default()
    }        
}

pub fn create_artist_credit() -> ArtistCredit {
    let artist = create_artist();
    ArtistCredit {
        name: artist.name.clone(),
        artist: artist.clone(),
        ..Default::default()
    }
}

pub fn create_artist_credits(min: usize, max: usize) -> Vec<ArtistCredit> {
    let mut artist_credits = vec![];
    for _ in 0..fakeit::misc::random(min, max) {
        artist_credits.push(create_artist_credit());
    }
    artist_credits
}

pub fn create_release_group(release: Release) -> ReleaseGroup {
    ReleaseGroup {
        annotation: None,
        artist_credits: release.artist_credits.clone(),
        disambiguation: release.disambiguation.clone(),
        first_release_date: release.date.clone(),
        genres: release.genres.clone(),
        key: None,
        known_ids: create_known_ids(),
        links: create_links(fakeit::misc::random(0, 5)),
        primary_type: Some(get_option(PRIMARY_TYPE_OPTIONS)),
        secondary_types: get_options(SECONDARY_TYPE_OPTIONS, 0, 1).into_iter().collect(),
        summary: release.summary.clone(),
        title: release.title.clone(),
    }
}

pub fn create_release() -> Release {
    let mut release = Release {
        artist_credits: create_artist_credits(1, 5),
        barcode: Some(format!("{}{}", fakeit::address::zip(), fakeit::address::zip())),
        country: Some(fakeit::address::country_abr()),
        date: Some(fakeit::datetime::year()),
        disambiguation: None,
        genres: create_genres(fakeit::misc::random(1, 5)),
        media: create_media(1, 4),
        key: None,
        known_ids: create_known_ids(),
        links: create_links(fakeit::misc::random(0, 5)),
        packaging: Some(PACKAGING_OPTIONS.get(fakeit::misc::random(0, PACKAGING_OPTIONS.len() - 1)).unwrap().to_string()),
        quality: Some(get_option(DATA_QUALITY_OPTIONS)),            
        status: Some(STATUS_OPTIONS.get(fakeit::misc::random(0, STATUS_OPTIONS.len() - 1)).unwrap().to_string()),
        summary: Some(fakeit::hipster::paragraph(2, 2, 40, " ".to_string())),
        title: Some(fakeit::hipster::sentence(2)),
        release_group: ReleaseGroup::default(),
        // TODO I don't quite understand why Musicbrainz doesn't have secondary type on release, or
        // alternately, why they both have primary type.
        // TODO Stopping here, finish this up, and the others, working on librarian merge tests.
    };

    release.release_group = create_release_group(release.clone());

    release
}

pub fn create_media(min: usize, max: usize) -> Vec<Medium> {
    let mut media = vec![];
    let disc_count = fakeit::misc::random(min, max);
    for i in 0..disc_count {
        let tracks = create_tracks(0, 20);
        let medium = Medium {
            disc_count: Some(disc_count as u32),
            format: None,
            key: None,
            position: Some(i as u32 + 1),
            title: None,
            track_count: Some(tracks.len() as u32),
            tracks,
        };
        media.push(medium);
    }
    media
}

pub fn create_tracks(min: usize, max: usize) -> Vec<Track> {
    let mut tracks = vec![];
    for i in 0..fakeit::misc::random(min, max) {
        let recording = create_recording();
        tracks.push(Track {
            artist_credits: recording.artist_credits.clone(),
            key: None,
            length: recording.length,
            position: Some(i as u32 + 1),
            number: None,
            title: recording.title.clone(),
            genres: recording.genres.clone(),
            known_ids: create_known_ids(),
            recording: recording.clone(),
        });
    }
    tracks
}

pub fn create_recording() -> Recording {
    Recording {
        annotation: None,
        disambiguation: None,
        isrc: Some(format!("{}{}", fakeit::address::zip(), fakeit::address::zip())),
        artist_credits: create_artist_credits(1, 3),
        key: None,
        length: Some(fakeit::misc::random(1, 30 * 60)),
        title: Some(fakeit::hipster::sentence(5)),
        genres: create_genres(fakeit::misc::random(1, 5)),
        known_ids: create_known_ids(),
        summary: Some(fakeit::hipster::paragraph(2, 2, 40, " ".to_string())),
        links: create_links(4),
    }
}

pub fn get_option(options: &[&str]) -> String {
    let i = fakeit::misc::random(0, options.len() - 1);
    options[i].to_string()
}

pub fn get_options(options: &[&str], min: usize, max: usize) -> Vec<String> {
    let mut results = vec![];
    for _ in 0..fakeit::misc::random(min, max) {
        results.push(get_option(options));
    }
    results
}

// Note that these lists are non-exhaustive.
const DATA_QUALITY_OPTIONS: &[&str] = &["unknown", "normal", "low", "high"];
const PRIMARY_TYPE_OPTIONS: &[&str] = &["album", "single", "ep", "other"];
const SECONDARY_TYPE_OPTIONS: &[&str] = &["compilation", "soundtrack", "spokenword", "interview", "audiobook", "audio drama", "live", "remix", "DJ-mix", "Mixtape/street", "Demo", "Field recording"];
const STATUS_OPTIONS: &[&str] = &["official", "promotion", "bootleg", "pseudo-release", "withdrawn", "cancelled"];
const PACKAGING_OPTIONS: &[&str] = &["Book", "Box", "Digipak", "Jewel case", "Other", "Cardboard/Paper Sleeve"];
const GENRE_OPTIONS: &[&str] = &["k-pop", "pop", "ambient", "reggae", 
    "jazz", "r&b", "rock", "dance", "merengue", "zydeco", "soul", "techno", 
    "bossa nova", "blues", "latin", "trance", "samba", "alternative", 
    "house", "folk", "gospel", "grunge", "electronic", "reggaeton", 
    "bolero", "tango", "afrobeat", "dubstep", "cajun", "punk", "bluegrass", 
    "world", "disco", "country", "salsa", "hip-hop", "metal", "cumbia", 
    "swing", "indie", "opera", "industrial", "classical", "ska", "mariachi",
    "soundtrack", "flamenco", "new age", "funk", "drum and bass"];
