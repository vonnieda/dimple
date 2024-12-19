use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::model::{MediaFile, Track};

pub trait Merge {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized;
}

// impl Merge for Artist {
//     fn merge(l: Self, r: Self) -> Option<Self> {
//         Some(Self {
//             country: Option::merge(l.country, r.country)?,
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
//             key: Option::merge(l.key, r.key)?,
//             known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
//             links: HashSet::merge(l.links, r.links)?,
//             name: Option::merge(l.name, r.name)?,
//             summary: Option::merge(l.summary, r.summary)?,
//             genres: Vec::merge(l.genres, r.genres)?,
//             // TODO assumes r is the new value
//             saved: r.saved,
//         })
//     }
// }

// impl Merge for Genre {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
//             key: Option::merge(l.key, r.key)?,
//             known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
//             links: HashSet::merge(l.links, r.links)?,
//             name: Option::merge(l.name, r.name)?,
//             summary: Option::merge(l.summary, r.summary)?,
//         })
//     }
// }

// impl Merge for ArtistCredit {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             key: Option::merge(l.key, r.key)?,
//             name: Option::merge(l.name, r.name)?,
//             join_phrase: Option::merge(l.join_phrase, r.join_phrase)?,
//             artist: Artist::merge(l.artist, r.artist)?,
//         })
//     }
// }

// impl Merge for Medium {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             key: Option::merge(l.key, r.key)?,
//             title: Option::merge(l.title, r.title)?,
//             disc_count: Option::merge(l.disc_count, r.disc_count)?,
//             format: Option::merge(l.format, r.format)?,
//             position: Option::merge(l.position, r.position)?,
//             track_count: Option::merge(l.track_count, r.track_count)?,
//             tracks: Vec::merge(l.tracks, r.tracks)?,
//         })
//     }
// }

// impl Merge for Track {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             key: Option::merge(l.key, r.key)?,
//             title: Option::merge(l.title, r.title)?,
//             position: Option::merge(l.position, r.position)?,
//             artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
//             genres: Vec::merge(l.genres, r.genres)?,
//             known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
//             length: Option::merge(l.length, r.length)?,
//             number: Option::merge(l.number, r.number)?,
//             recording: Recording::merge(l.recording, r.recording)?,
//         })
//     }
// }

// impl Merge for Track {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             album: Option::merge(l.album, r.album)?,
//             artist: Option::merge(l.artist, r.artist)?,
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
//             download: l.download || r.download,
//             key: Option::merge(l.key, r.key)?,
//             length_ms: Option::merge(l.length_ms, r.length_ms)?,
//             liked: l.liked || r.liked,
//             lyrics: Option::merge(l.lyrics, r.lyrics)?,
//             musicbrainz_id: Option::merge(l.musicbrainz_id, r.musicbrainz_id)?,
//             plays: l.plays.max(r.plays),
//             save: l.save || r.save,
//             spotify_id: Option::merge(l.spotify_id, r.spotify_id)?,
//             summary: Option::merge(l.summary, r.summary)?,
//             synced_lyrics: Option::merge(l.synced_lyrics, r.synced_lyrics)?,
//             title: Option::merge(l.title, r.title)?,
//             wikidata_id: Option::merge(l.wikidata_id, r.wikidata_id)?,
//         })
//     }
// }

// impl Merge for Track {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             album: l.album.or(r.album),
//             artist: l.artist.or(r.artist),
//             disambiguation: l.disambiguation.or(r.disambiguation),
//             download: l.download || r.download,
//             key: l.key.or(r.key),
//             length_ms: l.length_ms.or(r.length_ms),
//             liked: l.liked || r.liked,
//             lyrics: l.lyrics.or(r.lyrics),
//             musicbrainz_id: l.musicbrainz_id.or(r.musicbrainz_id),
//             plays: l.plays.max(r.plays),
//             save: l.save || r.save,
//             spotify_id: l.spotify_id.or(r.spotify_id),
//             summary: l.summary.or(r.summary),
//             synced_lyrics: l.synced_lyrics.or(r.synced_lyrics),
//             title: l.title.or(r.title),
//             wikidata_id: l.wikidata_id.or(r.wikidata_id),
//         })
//     }
// }

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

impl CrdtRules for Track {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            album: CrdtRules::merge(l.album, r.album),
            artist: CrdtRules::merge(l.artist, r.artist),
            disambiguation: CrdtRules::merge(l.disambiguation, r.disambiguation),
            download: CrdtRules::merge(l.download, r.download),
            key: CrdtRules::merge(l.key, r.key),
            length_ms: CrdtRules::merge(l.length_ms, r.length_ms),
            liked: CrdtRules::merge(l.liked, r.liked),
            lyrics: CrdtRules::merge(l.lyrics, r.lyrics),
            musicbrainz_id: CrdtRules::merge(l.musicbrainz_id, r.musicbrainz_id),
            plays: CrdtRules::merge(l.plays, r.plays),
            save: CrdtRules::merge(l.save, r.save),
            spotify_id: CrdtRules::merge(l.spotify_id, r.spotify_id),
            summary: CrdtRules::merge(l.summary, r.summary),
            synced_lyrics: CrdtRules::merge(l.synced_lyrics, r.synced_lyrics),
            title: CrdtRules::merge(l.title, r.title),
            wikidata_id: CrdtRules::merge(l.wikidata_id, r.wikidata_id),
            discogs_id: CrdtRules::merge(l.discogs_id, r.discogs_id),
            lastfm_id: CrdtRules::merge(l.lastfm_id, r.lastfm_id),
            media_position: CrdtRules::merge(l.media_position, r.media_position),
        }
    }
}

impl CrdtRules for MediaFile {
    fn merge(l: Self, r: Self) -> Self {
        Self {
            album: CrdtRules::merge(l.album, r.album),
            artist: CrdtRules::merge(l.artist, r.artist),
            file_path: CrdtRules::merge(l.file_path, r.file_path),
            genre: CrdtRules::merge(l.genre, r.genre),
            key: CrdtRules::merge(l.key, r.key),
            last_imported: CrdtRules::merge(l.last_imported, r.last_imported),
            last_modified: CrdtRules::merge(l.last_modified, r.last_modified),
            length_ms: CrdtRules::merge(l.length_ms, r.length_ms),
            lyrics: CrdtRules::merge(l.lyrics, r.lyrics),
            musicbrainz_album_artist_id: CrdtRules::merge(l.musicbrainz_album_artist_id, r.musicbrainz_album_artist_id),
            musicbrainz_album_id: CrdtRules::merge(l.musicbrainz_album_id, r.musicbrainz_album_id),
            musicbrainz_artist_id: CrdtRules::merge(l.musicbrainz_artist_id, r.musicbrainz_artist_id),
            musicbrainz_genre_id: CrdtRules::merge(l.musicbrainz_genre_id, r.musicbrainz_genre_id),
            musicbrainz_recording_id: CrdtRules::merge(l.musicbrainz_recording_id, r.musicbrainz_recording_id),
            musicbrainz_release_group_id: CrdtRules::merge(l.musicbrainz_release_group_id, r.musicbrainz_release_group_id),
            musicbrainz_release_track_id: CrdtRules::merge(l.musicbrainz_release_track_id, r.musicbrainz_release_track_id),
            musicbrainz_track_id: CrdtRules::merge(l.musicbrainz_track_id, r.musicbrainz_track_id),
            title: CrdtRules::merge(l.title, r.title),
            sha256: CrdtRules::merge(l.sha256, r.sha256),
            synced_lyrics: CrdtRules::merge(l.synced_lyrics, r.synced_lyrics),
            disc_number: CrdtRules::merge(l.disc_number, r.disc_number),
            disc_subtitle: CrdtRules::merge(l.disc_subtitle, r.disc_subtitle),
            isrc: CrdtRules::merge(l.isrc, r.isrc),
            label: CrdtRules::merge(l.label, r.label),
            original_date: CrdtRules::merge(l.original_date, r.original_date),
            original_year: CrdtRules::merge(l.original_year, r.original_year),
            release_date: CrdtRules::merge(l.release_date, r.release_date),
            total_discs: CrdtRules::merge(l.total_discs, r.total_discs),
            total_tracks: CrdtRules::merge(l.total_tracks, r.total_tracks),
            track_number: CrdtRules::merge(l.track_number, r.track_number),
            website: CrdtRules::merge(l.website, r.website),
        }
    }
}

impl CrdtRules for DateTime<Utc> {
    fn merge(l: Self, r: Self) -> Self {
        l.max(r)
    }
}

// impl Merge for ReleaseGroup {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             key: Option::merge(l.key, r.key)?,
//             title: Option::merge(l.title, r.title)?,
//             artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
//             genres: Vec::merge(l.genres, r.genres)?,
//             known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
//             links: HashSet::merge(l.links, r.links)?,
//             summary: Option::merge(l.summary, r.summary)?,
//             annotation: Option::merge(l.annotation, r.annotation)?,            
//             first_release_date: Option::merge(l.first_release_date, r.first_release_date)?,
//             primary_type: Option::merge(l.primary_type, r.primary_type)?,
//             secondary_types: HashSet::merge(l.secondary_types, r.secondary_types)?,
//         })
//     }
// }

// impl Merge for Release {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             key: Option::merge(l.key, r.key)?,
//             title: Option::merge(l.title, r.title)?,
//             artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
//             genres: Vec::merge(l.genres, r.genres)?,
//             known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
//             links: HashSet::merge(l.links, r.links)?,
//             summary: Option::merge(l.summary, r.summary)?,
//             barcode: Option::merge(l.barcode, r.barcode)?,
//             country: Option::merge(l.country, r.country)?,
//             date: Option::merge(l.date, r.date)?,
//             packaging: Option::merge(l.packaging, r.packaging)?,
//             quality: Option::merge(l.quality, r.quality)?,
//             status: Option::merge(l.status, r.status)?,
//             media: Vec::merge(l.media, r.media)?,
//             release_group: ReleaseGroup::merge(l.release_group, r.release_group)?,
//         })
//     }
// }

// impl Merge for Recording {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(Self {
//             key: Option::merge(l.key, r.key)?,
//             title: Option::merge(l.title, r.title)?,
//             artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
//             genres: Vec::merge(l.genres, r.genres)?,
//             known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
//             length: Option::merge(l.length, r.length)?,
//             disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
//             links: HashSet::merge(l.links, r.links)?,
//             summary: Option::merge(l.summary, r.summary)?,
//             annotation: Option::merge(l.annotation, r.annotation)?,
//             isrc: Option::merge(l.isrc, r.isrc)?,
//         })
//     }
// }

// impl Merge for String {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         if l == r {
//             Some(l)
//         }
//         else {
//             None
//         }
//     }
// }

// impl Merge for u32 {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         if l == r {
//             Some(l)
//         }
//         else {
//             None
//         }
//     }
// }

// impl Merge for u64 {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         if l == r {
//             Some(l)
//         }
//         else {
//             None
//         }
//     }
// }

// impl Merge for bool {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         if l == r {
//             Some(l)
//         }
//         else {
//             None
//         }
//     }
// }

// impl <T: Merge> Merge for Option<T> {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         match (l, r) {
//             (Some(l), Some(r)) => Some(Some(Merge::merge(l, r)?)),
//             (None, None) => Some(None),
//             (None, Some(r)) => Some(Some(r)),
//             (Some(l), None) => Some(Some(l)),
//         }
//     }
// }

// impl Merge for KnownIds {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(KnownIds {
//             musicbrainz_id: Option::merge(l.musicbrainz_id, r.musicbrainz_id)?,
//             discogs_id: Option::merge(l.discogs_id, r.discogs_id)?,
//             lastfm_id: Option::merge(l.lastfm_id, r.lastfm_id)?,
//         })
//     }
// }

// impl Merge for HashSet<String> {
//     fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
//         Some(l.union(&r).cloned().collect())
//     }
// }

// impl <T: Merge + Clone> Merge for Vec<T> {
//     fn merge(l: Self, r: Self) -> Option<Self> {
//         let mut result = l.clone();
    
//         for b in r {
//             let mut merged = false;
    
//             for a in &mut result {
//                 let m = T::merge(a.clone(), b.clone());
//                 if m.is_some() {
//                     *a = m.unwrap();
//                     merged = true;
//                     break;
//                 }
//             }
    
//             if !merged {
//                 result.push(b);
//             }
//         }
    
//         Some(result)
//     }
// }

// impl Merge for Model {
//     fn merge(l: Self, r: Self) -> Option<Self> {
//         match (l, r) {
//             (Model::Artist(l), Model::Artist(r)) => Some(Artist::merge(l.clone(), r.clone())?.model()),
//             (Model::Release(l), Model::Release(r)) => Some(Release::merge(l.clone(), r.clone())?.model()),
//             (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => Some(ReleaseGroup::merge(l.clone(), r.clone())?.model()),
//             (Model::Recording(l), Model::Recording(r)) => Some(Recording::merge(l.clone(), r.clone())?.model()),
//             (Model::Genre(l), Model::Genre(r)) => Some(Genre::merge(l.clone(), r.clone())?.model()),
//             // (Model::Medium(l), Model::Medium(r)) => Some(Medium::nu_merge(l.clone(), r.clone()).model(),
//             // (Model::Track(l), Model::Track(r)) => Some(Track::nu_merge(l.clone(), r.clone()).model(),
//             // (Model::Dimage(l), Model::Dimage(r)) => Dimage::nu_merge(l.clone(), r.clone()).model(),
//             _ => todo!()
//         }
//     }
// }

#[cfg(test)]
mod test {
    use crate::{merge::CrdtRules, model::Track};
    use super::Merge;

    #[test]
    fn is_crdt() {
        let a = Track {
            album: Some("Ride the Lightning".to_string()),
            artist: Some("Metallica".to_string()),
            disambiguation: None,
            download: false,
            key: Some("1140b370-dccd-4087-854f-926d8798b552".to_string()),
            length_ms: Some(1000 * 60 * 3),
            liked: true,
            lyrics: Some("You gotta ride that lightning. I'm telling you.".to_string()),
            musicbrainz_id: Some("454adf09-f92b-4e57-b099-dba4420823a8".to_string()),
            plays: 100,
            save: true,
            spotify_id: None,
            summary: Some("Metallica's first ever track, written in honor of their third son.".to_string()),
            synced_lyrics: None,
            title: Some("Ride the Lightning".to_string()),
            wikidata_id: Some("Q0123129".to_string()),
            ..Default::default()
        };
        let b = Track {
            album: Some("Ride the Lightning".to_string()),
            artist: Some("Metallica".to_string()),
            disambiguation: None,
            download: false,
            key: Some("1140b370-dccd-4087-854f-926d8798b552".to_string()),
            length_ms: Some(1000 * 60 * 3),
            liked: true,
            lyrics: Some("You gotta ride that lightning. I'm telling you.".to_string()),
            musicbrainz_id: Some("454adf09-f92b-4e57-b099-dba4420823a8".to_string()),
            plays: 100,
            save: true,
            spotify_id: None,
            summary: Some("Metallica's first ever track, written in honor of their second son.".to_string()),
            synced_lyrics: None,
            title: Some("Ride the Lightning".to_string()),
            wikidata_id: Some("Q0123129".to_string()),
            ..Default::default()
        };
        let c = Track {
            album: Some("Ride the Hip-hop".to_string()),
            artist: Some("Metallica".to_string()),
            disambiguation: Some("Totally cool".to_string()),
            download: true,
            key: Some("1140b370-dccd-4087-854f-926d8798b552".to_string()),
            length_ms: Some(1000 * 60 * 3),
            liked: true,
            lyrics: Some("You gotta ride that lightning. I'm telling you. Welcome to the Lightning, baby.".to_string()),
            musicbrainz_id: Some("454adf09-f92b-4e57-b099-dba4420823a8".to_string()),
            plays: 100,
            save: true,
            spotify_id: None,
            summary: Some("Metallica's first ever track, written in honor of their first son.".to_string()),
            synced_lyrics: None,
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