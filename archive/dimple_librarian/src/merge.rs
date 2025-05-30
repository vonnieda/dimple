use std::collections::HashSet;

use dimple_core::model::{Artist, ArtistCredit, Entity, Genre, KnownIds, Medium, Model, Recording, Release, ReleaseGroup, Track};

pub trait Merge {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized;
}

impl Merge for Artist {
    fn merge(l: Self, r: Self) -> Option<Self> {
        Some(Self {
            country: Option::merge(l.country, r.country)?,
            disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
            key: Option::merge(l.key, r.key)?,
            known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
            links: HashSet::merge(l.links, r.links)?,
            name: Option::merge(l.name, r.name)?,
            summary: Option::merge(l.summary, r.summary)?,
            genres: Vec::merge(l.genres, r.genres)?,
            // TODO assumes r is the new value
            saved: r.saved,
        })
    }
}

impl Merge for Genre {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
            key: Option::merge(l.key, r.key)?,
            known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
            links: HashSet::merge(l.links, r.links)?,
            name: Option::merge(l.name, r.name)?,
            summary: Option::merge(l.summary, r.summary)?,
        })
    }
}

impl Merge for ArtistCredit {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::merge(l.key, r.key)?,
            name: Option::merge(l.name, r.name)?,
            join_phrase: Option::merge(l.join_phrase, r.join_phrase)?,
            artist: Artist::merge(l.artist, r.artist)?,
        })
    }
}

impl Merge for Medium {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::merge(l.key, r.key)?,
            title: Option::merge(l.title, r.title)?,
            disc_count: Option::merge(l.disc_count, r.disc_count)?,
            format: Option::merge(l.format, r.format)?,
            position: Option::merge(l.position, r.position)?,
            track_count: Option::merge(l.track_count, r.track_count)?,
            tracks: Vec::merge(l.tracks, r.tracks)?,
        })
    }
}

impl Merge for Track {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::merge(l.key, r.key)?,
            title: Option::merge(l.title, r.title)?,
            position: Option::merge(l.position, r.position)?,
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::merge(l.genres, r.genres)?,
            known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
            length: Option::merge(l.length, r.length)?,
            number: Option::merge(l.number, r.number)?,
            recording: Recording::merge(l.recording, r.recording)?,
        })
    }
}

impl Merge for ReleaseGroup {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::merge(l.key, r.key)?,
            title: Option::merge(l.title, r.title)?,
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::merge(l.genres, r.genres)?,
            known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
            disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
            links: HashSet::merge(l.links, r.links)?,
            summary: Option::merge(l.summary, r.summary)?,
            annotation: Option::merge(l.annotation, r.annotation)?,            
            first_release_date: Option::merge(l.first_release_date, r.first_release_date)?,
            primary_type: Option::merge(l.primary_type, r.primary_type)?,
            secondary_types: HashSet::merge(l.secondary_types, r.secondary_types)?,
        })
    }
}

impl Merge for Release {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::merge(l.key, r.key)?,
            title: Option::merge(l.title, r.title)?,
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::merge(l.genres, r.genres)?,
            known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
            disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
            links: HashSet::merge(l.links, r.links)?,
            summary: Option::merge(l.summary, r.summary)?,
            barcode: Option::merge(l.barcode, r.barcode)?,
            country: Option::merge(l.country, r.country)?,
            date: Option::merge(l.date, r.date)?,
            packaging: Option::merge(l.packaging, r.packaging)?,
            quality: Option::merge(l.quality, r.quality)?,
            status: Option::merge(l.status, r.status)?,
            media: Vec::merge(l.media, r.media)?,
            release_group: ReleaseGroup::merge(l.release_group, r.release_group)?,
        })
    }
}

impl Merge for Recording {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(Self {
            key: Option::merge(l.key, r.key)?,
            title: Option::merge(l.title, r.title)?,
            artist_credits: Vec::merge(l.artist_credits, r.artist_credits)?,
            genres: Vec::merge(l.genres, r.genres)?,
            known_ids: KnownIds::merge(l.known_ids, r.known_ids)?,
            length: Option::merge(l.length, r.length)?,
            disambiguation: Option::merge(l.disambiguation, r.disambiguation)?,
            links: HashSet::merge(l.links, r.links)?,
            summary: Option::merge(l.summary, r.summary)?,
            annotation: Option::merge(l.annotation, r.annotation)?,
            isrc: Option::merge(l.isrc, r.isrc)?,
        })
    }
}

impl Merge for String {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        if l == r {
            Some(l)
        }
        else {
            None
        }
    }
}

impl Merge for u32 {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        if l == r {
            Some(l)
        }
        else {
            None
        }
    }
}

impl <T: Merge> Merge for Option<T> {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        match (l, r) {
            (Some(l), Some(r)) => Some(Some(Merge::merge(l, r)?)),
            (None, None) => Some(None),
            (None, Some(r)) => Some(Some(r)),
            (Some(l), None) => Some(Some(l)),
        }
    }
}

impl Merge for KnownIds {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(KnownIds {
            musicbrainz_id: Option::merge(l.musicbrainz_id, r.musicbrainz_id)?,
            discogs_id: Option::merge(l.discogs_id, r.discogs_id)?,
            lastfm_id: Option::merge(l.lastfm_id, r.lastfm_id)?,
        })
    }
}

impl Merge for HashSet<String> {
    fn merge(l: Self, r: Self) -> Option<Self> where Self: Sized {
        Some(l.union(&r).cloned().collect())
    }
}

impl <T: Merge + Clone> Merge for Vec<T> {
    fn merge(l: Self, r: Self) -> Option<Self> {
        let mut result = l.clone();
    
        for b in r {
            let mut merged = false;
    
            for a in &mut result {
                let m = T::merge(a.clone(), b.clone());
                if m.is_some() {
                    *a = m.unwrap();
                    merged = true;
                    break;
                }
            }
    
            if !merged {
                result.push(b);
            }
        }
    
        Some(result)
    }
}

impl Merge for Model {
    fn merge(l: Self, r: Self) -> Option<Self> {
        match (l, r) {
            (Model::Artist(l), Model::Artist(r)) => Some(Artist::merge(l.clone(), r.clone())?.model()),
            (Model::Release(l), Model::Release(r)) => Some(Release::merge(l.clone(), r.clone())?.model()),
            (Model::ReleaseGroup(l), Model::ReleaseGroup(r)) => Some(ReleaseGroup::merge(l.clone(), r.clone())?.model()),
            (Model::Recording(l), Model::Recording(r)) => Some(Recording::merge(l.clone(), r.clone())?.model()),
            (Model::Genre(l), Model::Genre(r)) => Some(Genre::merge(l.clone(), r.clone())?.model()),
            // (Model::Medium(l), Model::Medium(r)) => Some(Medium::nu_merge(l.clone(), r.clone()).model(),
            // (Model::Track(l), Model::Track(r)) => Some(Track::nu_merge(l.clone(), r.clone()).model(),
            // (Model::Dimage(l), Model::Dimage(r)) => Dimage::nu_merge(l.clone(), r.clone()).model(),
            _ => todo!()
        }
    }
}
