use crate::model::{Artist, Dimage, Genre, Link, Release, ReleaseGroup, Track};

pub trait MergeExtend {
    /// Extend the object by filling in missing values from the other object.
    fn merge_extend(&self, other: &Self) -> Self;
}

impl MergeExtend for Option<String> {
    fn merge_extend(&self, other: &Self) -> Self {
        self.clone().or(other.clone())
    }
}

impl MergeExtend for Option<u64> {
    fn merge_extend(&self, other: &Self) -> Self {
        self.or(*other)
    }
}

impl MergeExtend for Option<u32> {
    fn merge_extend(&self, other: &Self) -> Self {
        self.or(*other)
    }
}

impl MergeExtend for Artist {
    fn merge_extend(&self, other: &Self) -> Self {
        Artist {
            id: self.id.merge_extend(&other.id),
            name: self.name.merge_extend(&other.name),
            disambiguation: self.disambiguation.merge_extend(&other.disambiguation),
            summary: self.summary.merge_extend(&other.summary),
            save: self.save,
            download: self.download,
            country: self.country.merge_extend(&other.country),
            discogs_id: self.discogs_id.merge_extend(&other.discogs_id),
            lastfm_id: self.lastfm_id.merge_extend(&other.lastfm_id),
            musicbrainz_id: self.musicbrainz_id.merge_extend(&other.musicbrainz_id),
            spotify_id: self.spotify_id.merge_extend(&other.spotify_id),
            wikidata_id: self.wikidata_id.merge_extend(&other.wikidata_id),
        }
    }
}

impl MergeExtend for ReleaseGroup {
    fn merge_extend(&self, other: &Self) -> Self {
        ReleaseGroup {
            id: self.id.merge_extend(&other.id),
            title: self.title.merge_extend(&other.title),
            disambiguation: self.disambiguation.merge_extend(&other.disambiguation),
            summary: self.summary.merge_extend(&other.summary),
            save: self.save,
            download: self.download,
            first_release_date: self.first_release_date.merge_extend(&other.first_release_date),
            primary_type: self.primary_type.clone().or(other.primary_type.clone()),
            discogs_id: self.discogs_id.merge_extend(&other.discogs_id),
            lastfm_id: self.lastfm_id.merge_extend(&other.lastfm_id),
            musicbrainz_id: self.musicbrainz_id.merge_extend(&other.musicbrainz_id),
            spotify_id: self.spotify_id.merge_extend(&other.spotify_id),
            wikidata_id: self.wikidata_id.merge_extend(&other.wikidata_id),
        }
    }
}

impl MergeExtend for Release {
    fn merge_extend(&self, other: &Self) -> Self {
        Release {
            id: self.id.merge_extend(&other.id),
            title: self.title.merge_extend(&other.title),
            disambiguation: self.disambiguation.merge_extend(&other.disambiguation),
            summary: self.summary.merge_extend(&other.summary),
            save: self.save,
            download: self.download,
            barcode: self.barcode.merge_extend(&other.barcode),
            country: self.country.merge_extend(&other.country),
            date: self.date.merge_extend(&other.date),
            packaging: self.packaging.merge_extend(&other.packaging),
            status: self.status.merge_extend(&other.status),
            quality: self.quality.merge_extend(&other.quality),
            release_group_id: self.release_group_id.merge_extend(&other.release_group_id),
            discogs_id: self.discogs_id.merge_extend(&other.discogs_id),
            lastfm_id: self.lastfm_id.merge_extend(&other.lastfm_id),
            musicbrainz_id: self.musicbrainz_id.merge_extend(&other.musicbrainz_id),
            spotify_id: self.spotify_id.merge_extend(&other.spotify_id),
            wikidata_id: self.wikidata_id.merge_extend(&other.wikidata_id),
        }
    }
}

impl MergeExtend for Track {
    fn merge_extend(&self, other: &Self) -> Self {
        Track {
            id: self.id.merge_extend(&other.id),
            title: self.title.merge_extend(&other.title),
            disambiguation: self.disambiguation.merge_extend(&other.disambiguation),
            summary: self.summary.merge_extend(&other.summary),
            save: self.save,
            download: self.download,
            discogs_id: self.discogs_id.merge_extend(&other.discogs_id),
            lastfm_id: self.lastfm_id.merge_extend(&other.lastfm_id),
            musicbrainz_id: self.musicbrainz_id.merge_extend(&other.musicbrainz_id),
            spotify_id: self.spotify_id.merge_extend(&other.spotify_id),
            wikidata_id: self.wikidata_id.merge_extend(&other.wikidata_id),
            release_id: self.release_id.merge_extend(&other.release_id),
            position: self.position.merge_extend(&other.position),
            length_ms: self.length_ms.merge_extend(&other.length_ms),
            lyrics: self.lyrics.merge_extend(&other.lyrics),
            synchronized_lyrics: self.synchronized_lyrics.merge_extend(&other.synchronized_lyrics),
            media_track_count: self.media_track_count.merge_extend(&other.media_track_count),
            media_position: self.media_position.merge_extend(&other.media_position),
            media_title: self.media_title.merge_extend(&other.media_title),
            media_format: self.media_format.merge_extend(&other.media_format),
        }
    }
}

impl MergeExtend for Genre {
    fn merge_extend(&self, other: &Self) -> Self {
        Genre {
            id: self.id.merge_extend(&other.id),
            name: self.name.merge_extend(&other.name),
            disambiguation: self.disambiguation.merge_extend(&other.disambiguation),
            summary: self.summary.merge_extend(&other.summary),
            save: self.save,
            download: self.download,
            discogs_id: self.discogs_id.merge_extend(&other.discogs_id),
            lastfm_id: self.lastfm_id.merge_extend(&other.lastfm_id),
            musicbrainz_id: self.musicbrainz_id.merge_extend(&other.musicbrainz_id),
            spotify_id: self.spotify_id.merge_extend(&other.spotify_id),
            wikidata_id: self.wikidata_id.merge_extend(&other.wikidata_id),
        }
    }
}

impl MergeExtend for Link {
    fn merge_extend(&self, other: &Self) -> Self {
        Link {
            id: self.id.merge_extend(&other.id),
            name: self.name.merge_extend(&other.name),
            url: if self.url.is_empty() { other.url.clone() } else { self.url.clone() },
        }
    }
}

impl MergeExtend for Dimage {
    fn merge_extend(&self, other: &Self) -> Self {
        Dimage {
            id: self.id.merge_extend(&other.id),
            kind: self.kind.clone().or(other.kind.clone()),
            width: if self.width == 0 { other.width } else { self.width },
            height: if self.height == 0 { other.height } else { self.height },
            png_thumbnail: if self.png_thumbnail.is_empty() { other.png_thumbnail.clone() } else { self.png_thumbnail.clone() },
            png_data: if self.png_data.is_empty() { other.png_data.clone() } else { self.png_data.clone() },
            sha256: if self.sha256.is_empty() { other.sha256.clone() } else { self.sha256.clone() },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge_extend_link() {
        let link1 = Link {
            id: Some("link-1".to_string()),
            name: Some("Official Site".to_string()),
            url: "https://example.com".to_string(),
        };

        let link2 = Link {
            id: None,
            name: None,
            url: "https://other.com".to_string(),
        };

        let merged = link1.merge_extend(&link2);
        assert_eq!(merged.id, Some("link-1".to_string()));
        assert_eq!(merged.name, Some("Official Site".to_string()));
        assert_eq!(merged.url, "https://example.com".to_string());
    }

    #[test]
    fn test_merge_extend_link_empty_url() {
        let link1 = Link {
            id: Some("link-1".to_string()),
            name: None,
            url: "".to_string(),
        };

        let link2 = Link {
            id: None,
            name: Some("Wikipedia".to_string()),
            url: "https://wikipedia.org".to_string(),
        };

        let merged = link1.merge_extend(&link2);
        assert_eq!(merged.name, Some("Wikipedia".to_string()));
        assert_eq!(merged.url, "https://wikipedia.org".to_string());
    }

    #[test]
    fn test_merge_extend_genre() {
        let genre1 = Genre {
            id: Some("genre-1".to_string()),
            name: Some("Rock".to_string()),
            musicbrainz_id: Some("mbid-123".to_string()),
            ..Default::default()
        };

        let genre2 = Genre {
            name: Some("Rock".to_string()),
            disambiguation: Some("rock music".to_string()),
            summary: Some("A genre of popular music".to_string()),
            ..Default::default()
        };

        let merged = genre1.merge_extend(&genre2);
        assert_eq!(merged.id, Some("genre-1".to_string()));
        assert_eq!(merged.name, Some("Rock".to_string()));
        assert_eq!(merged.musicbrainz_id, Some("mbid-123".to_string()));
        assert_eq!(merged.disambiguation, Some("rock music".to_string()));
        assert_eq!(merged.summary, Some("A genre of popular music".to_string()));
    }

    #[test]
    fn test_merge_extend_dimage() {
        let dimage1 = Dimage {
            id: Some("img-1".to_string()),
            sha256: "abc123".to_string(),
            width: 500,
            height: 500,
            ..Default::default()
        };

        let dimage2 = Dimage {
            width: 1000,
            height: 1000,
            png_data: vec![1, 2, 3, 4],
            sha256: "def456".to_string(),
            ..Default::default()
        };

        let merged = dimage1.merge_extend(&dimage2);
        assert_eq!(merged.id, Some("img-1".to_string()));
        assert_eq!(merged.sha256, "abc123".to_string());
        assert_eq!(merged.width, 500);
        assert_eq!(merged.height, 500);
        assert_eq!(merged.png_data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_merge_extend_dimage_empty() {
        let dimage1 = Dimage {
            id: Some("img-1".to_string()),
            ..Default::default()
        };

        let dimage2 = Dimage {
            width: 800,
            height: 600,
            sha256: "xyz789".to_string(),
            ..Default::default()
        };

        let merged = dimage1.merge_extend(&dimage2);
        assert_eq!(merged.width, 800);
        assert_eq!(merged.height, 600);
        assert_eq!(merged.sha256, "xyz789".to_string());
    }
}
