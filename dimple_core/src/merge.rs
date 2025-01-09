use chrono::{DateTime, Utc};

use crate::model::{Artist, Genre, MediaFile, Release, Track};

pub trait CrdtRules {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn merge(l: Self, r: Self) -> Self;
}

impl CrdtRules for String {
    fn merge(l: Self, r: Self) -> Self {
        if l.len() >= r.len() {
            l
        }
        else {
            r
        }
    }
}

impl CrdtRules for bool {
    fn merge(l: Self, r: Self) -> Self {
        l || r
    }
}

impl CrdtRules for u32 {
    fn merge(l: Self, r: Self) -> Self {
        l.max(r)
    }
}

impl CrdtRules for u64 {
    fn merge(l: Self, r: Self) -> Self {
        l.max(r)
    }
}

impl <T> CrdtRules for Option<T> where T: CrdtRules {
    fn merge(l: Self, r: Self) -> Self {
        if l.is_some() && r.is_some() {
            Some(CrdtRules::merge(l.unwrap(), r.unwrap()))
        }
        else {
            l.or(r)
        }
    }
}

impl CrdtRules for Artist {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: CrdtRules::merge(l.key, r.key),
            name: CrdtRules::merge(l.name, r.name),
            disambiguation: CrdtRules::merge(l.disambiguation, r.disambiguation),
            summary: CrdtRules::merge(l.summary, r.summary),
            save: CrdtRules::merge(l.save, r.save),
            download: CrdtRules::merge(l.download, r.download),
            
            // release_key: CrdtRules::merge(l.release_key, r.release_key),
            // position: CrdtRules::merge(l.position, r.position),
            // length_ms: CrdtRules::merge(l.length_ms, r.length_ms),
            // lyrics: CrdtRules::merge(l.lyrics, r.lyrics),
            // synchronized_lyrics: CrdtRules::merge(l.synchronized_lyrics, r.synchronized_lyrics),

            // barcode: CrdtRules::merge(l.barcode, r.barcode),
            country: CrdtRules::merge(l.country, r.country),
            // date: CrdtRules::merge(l.date, r.date),
            // packaging: CrdtRules::merge(l.packaging, r.packaging),
            // status: CrdtRules::merge(l.status, r.status),
            // quality: CrdtRules::merge(l.quality, r.quality),
            // release_group_type: CrdtRules::merge(l.release_group_type, r.release_group_type),

            discogs_id: CrdtRules::merge(l.discogs_id, r.discogs_id),
            lastfm_id: CrdtRules::merge(l.lastfm_id, r.lastfm_id),
            musicbrainz_id: CrdtRules::merge(l.musicbrainz_id, r.musicbrainz_id),
            spotify_id: CrdtRules::merge(l.spotify_id, r.spotify_id),
            wikidata_id: CrdtRules::merge(l.wikidata_id, r.wikidata_id),

            // media_format: CrdtRules::merge(l.media_format, r.media_format),
            // media_position: CrdtRules::merge(l.media_position, r.media_position),
            // media_title: CrdtRules::merge(l.media_title, r.media_title),
            // media_track_count: CrdtRules::merge(l.media_track_count, r.media_track_count),            
        }
    }
}

impl CrdtRules for Release {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: CrdtRules::merge(l.key, r.key),
            title: CrdtRules::merge(l.title, r.title),
            disambiguation: CrdtRules::merge(l.disambiguation, r.disambiguation),
            summary: CrdtRules::merge(l.summary, r.summary),
            save: CrdtRules::merge(l.save, r.save),
            download: CrdtRules::merge(l.download, r.download),
            
            // release_key: CrdtRules::merge(l.release_key, r.release_key),
            // position: CrdtRules::merge(l.position, r.position),
            // length_ms: CrdtRules::merge(l.length_ms, r.length_ms),
            // lyrics: CrdtRules::merge(l.lyrics, r.lyrics),
            // synchronized_lyrics: CrdtRules::merge(l.synchronized_lyrics, r.synchronized_lyrics),

            barcode: CrdtRules::merge(l.barcode, r.barcode),
            country: CrdtRules::merge(l.country, r.country),
            date: CrdtRules::merge(l.date, r.date),
            packaging: CrdtRules::merge(l.packaging, r.packaging),
            status: CrdtRules::merge(l.status, r.status),
            quality: CrdtRules::merge(l.quality, r.quality),
            release_group_type: CrdtRules::merge(l.release_group_type, r.release_group_type),

            discogs_id: CrdtRules::merge(l.discogs_id, r.discogs_id),
            lastfm_id: CrdtRules::merge(l.lastfm_id, r.lastfm_id),
            musicbrainz_id: CrdtRules::merge(l.musicbrainz_id, r.musicbrainz_id),
            spotify_id: CrdtRules::merge(l.spotify_id, r.spotify_id),
            wikidata_id: CrdtRules::merge(l.wikidata_id, r.wikidata_id),

            // media_format: CrdtRules::merge(l.media_format, r.media_format),
            // media_position: CrdtRules::merge(l.media_position, r.media_position),
            // media_title: CrdtRules::merge(l.media_title, r.media_title),
            // media_track_count: CrdtRules::merge(l.media_track_count, r.media_track_count),            
        }
    }
}

impl CrdtRules for Track {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: CrdtRules::merge(l.key, r.key),
            title: CrdtRules::merge(l.title, r.title),
            disambiguation: CrdtRules::merge(l.disambiguation, r.disambiguation),
            summary: CrdtRules::merge(l.summary, r.summary),
            save: CrdtRules::merge(l.save, r.save),
            download: CrdtRules::merge(l.download, r.download),
            
            release_key: CrdtRules::merge(l.release_key, r.release_key),
            position: CrdtRules::merge(l.position, r.position),
            length_ms: CrdtRules::merge(l.length_ms, r.length_ms),
            lyrics: CrdtRules::merge(l.lyrics, r.lyrics),
            synchronized_lyrics: CrdtRules::merge(l.synchronized_lyrics, r.synchronized_lyrics),

            discogs_id: CrdtRules::merge(l.discogs_id, r.discogs_id),
            lastfm_id: CrdtRules::merge(l.lastfm_id, r.lastfm_id),
            musicbrainz_id: CrdtRules::merge(l.musicbrainz_id, r.musicbrainz_id),
            spotify_id: CrdtRules::merge(l.spotify_id, r.spotify_id),
            wikidata_id: CrdtRules::merge(l.wikidata_id, r.wikidata_id),

            media_format: CrdtRules::merge(l.media_format, r.media_format),
            media_position: CrdtRules::merge(l.media_position, r.media_position),
            media_title: CrdtRules::merge(l.media_title, r.media_title),
            media_track_count: CrdtRules::merge(l.media_track_count, r.media_track_count),
        }
    }
}

impl CrdtRules for MediaFile {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            file_path: CrdtRules::merge(l.file_path, r.file_path),
            key: CrdtRules::merge(l.key, r.key),
            last_imported: CrdtRules::merge(l.last_imported, r.last_imported),
            last_modified: CrdtRules::merge(l.last_modified, r.last_modified),
            sha256: CrdtRules::merge(l.sha256, r.sha256),
        }
    }
}

impl CrdtRules for Genre {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            key: CrdtRules::merge(l.key, r.key),
            name: CrdtRules::merge(l.name, r.name),
            disambiguation: CrdtRules::merge(l.disambiguation, r.disambiguation),
            summary: CrdtRules::merge(l.summary, r.summary),
            save: CrdtRules::merge(l.save, r.save),
            download: CrdtRules::merge(l.download, r.download),
            
            discogs_id: CrdtRules::merge(l.discogs_id, r.discogs_id),
            lastfm_id: CrdtRules::merge(l.lastfm_id, r.lastfm_id),
            musicbrainz_id: CrdtRules::merge(l.musicbrainz_id, r.musicbrainz_id),
            spotify_id: CrdtRules::merge(l.spotify_id, r.spotify_id),
            wikidata_id: CrdtRules::merge(l.wikidata_id, r.wikidata_id),
        }
    }
}

impl CrdtRules for DateTime<Utc> {
    fn merge(l: Self, r: Self) -> Self {
        l.max(r)
    }
}

#[cfg(test)]
mod test {
    use crate::{merge::CrdtRules, model::Track};

    #[test]
    fn is_crdt() {
        let a = Track {
            disambiguation: None,
            download: false,
            key: Some("1140b370-dccd-4087-854f-926d8798b552".to_string()),
            length_ms: Some(1000 * 60 * 3),
            lyrics: Some("You gotta ride that lightning. I'm telling you.".to_string()),
            musicbrainz_id: Some("454adf09-f92b-4e57-b099-dba4420823a8".to_string()),
            save: true,
            spotify_id: None,
            summary: Some("Metallica's first ever track, written in honor of their third son.".to_string()),
            synchronized_lyrics: None,
            title: Some("Ride the Lightning".to_string()),
            wikidata_id: Some("Q0123129".to_string()),
            ..Default::default()
        };
        let b = Track {
            disambiguation: None,
            download: false,
            key: Some("1140b370-dccd-4087-854f-926d8798b552".to_string()),
            length_ms: Some(1000 * 60 * 3),
            lyrics: Some("You gotta ride that lightning. I'm telling you.".to_string()),
            musicbrainz_id: Some("454adf09-f92b-4e57-b099-dba4420823a8".to_string()),
            save: true,
            spotify_id: None,
            summary: Some("Metallica's first ever track, written in honor of their second son.".to_string()),
            synchronized_lyrics: None,
            title: Some("Ride the Lightning".to_string()),
            wikidata_id: Some("Q0123129".to_string()),
            ..Default::default()
        };
        let c = Track {
            disambiguation: Some("Totally cool".to_string()),
            download: true,
            key: Some("1140b370-dccd-4087-854f-926d8798b552".to_string()),
            length_ms: Some(1000 * 60 * 3),
            lyrics: Some("You gotta ride that lightning. I'm telling you. Welcome to the Lightning, baby.".to_string()),
            musicbrainz_id: Some("454adf09-f92b-4e57-b099-dba4420823a8".to_string()),
            save: true,
            spotify_id: None,
            summary: Some("Metallica's first ever track, written in honor of their first son.".to_string()),
            synchronized_lyrics: None,
            title: Some("Ride the Lightning".to_string()),
            wikidata_id: Some("Q0123129".to_string()),
            ..Default::default()
        };
        /// Commutative: A v B = B v A
        /// Associative: (A v B) v C = A v (B v C)
        /// Idempotent : A v A = A
        assert!(CrdtRules::merge(a.clone(), b.clone()) == CrdtRules::merge(b.clone(), a.clone()));
        assert!(CrdtRules::merge(CrdtRules::merge(a.clone(), b.clone()), c.clone()) 
            == CrdtRules::merge(a.clone(), CrdtRules::merge(b.clone(), c.clone())));
        assert!(CrdtRules::merge(a.clone(), a.clone()) == a.clone());
    }
}