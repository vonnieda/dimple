
use thiserror::Error;

use crate::model::{Artist, Release, Track};


#[derive(Debug, Error)]
pub enum MergeError {
    #[error("Cannot merge: conflicting {field} values: '{left}' vs '{right}'")]
    ConflictingField {
        field: String,
        left: String,
        right: String,
    },

    #[error("Cannot merge: incompatible entities")]
    IncompatibleEntities(String),
}

pub trait MergeRules<T> {
    fn try_merge(l: &T, r: &T) -> Result<T, MergeError>;
}

fn merge_field<T: PartialEq + Clone + ToString>(
    field_name: &str,
    left: &Option<T>,
    right: &Option<T>,
) -> Result<Option<T>, MergeError> {
    match (left, right) {
        (Some(l_val), Some(r_val)) => {
            if l_val != r_val {
                Err(MergeError::ConflictingField {
                    field: field_name.to_string(),
                    left: l_val.to_string(),
                    right: r_val.to_string(),
                })
            } else {
                Ok(Some(l_val.clone()))
            }
        }
        (Some(val), None) => Ok(Some(val.clone())),
        (None, Some(val)) => Ok(Some(val.clone())),
        (None, None) => Ok(None),
    }
}

fn merge_field_case_insensitive(
    field_name: &str,
    left: &Option<String>,
    right: &Option<String>,
) -> Result<Option<String>, MergeError> {
    match (left, right) {
        (Some(l_val), Some(r_val)) => {
            if l_val.to_lowercase() != r_val.to_lowercase() {
                Err(MergeError::ConflictingField {
                    field: field_name.to_string(),
                    left: l_val.clone(),
                    right: r_val.clone(),
                })
            } else {
                // Prefer left value for case preservation
                Ok(Some(l_val.clone()))
            }
        }
        (Some(val), None) => Ok(Some(val.clone())),
        (None, Some(val)) => Ok(Some(val.clone())),
        (None, None) => Ok(None),
    }
}

impl MergeRules<Release> for Release {
    fn try_merge(l: &Release, r: &Release) -> Result<Release, MergeError> {
        Ok(Release {
            id: merge_field("id", &l.id, &r.id)?,
            title: merge_field_case_insensitive("title", &l.title, &r.title)?,
            disambiguation: merge_field("disambiguation", &l.disambiguation, &r.disambiguation)?,
            summary: merge_field("summary", &l.summary, &r.summary)?,
            save: l.save || r.save,
            download: l.download || r.download,
            
            barcode: merge_field("barcode", &l.barcode, &r.barcode)?,
            country: merge_field("country", &l.country, &r.country)?,
            date: merge_field("date", &l.date, &r.date)?,
            packaging: merge_field("packaging", &l.packaging, &r.packaging)?,
            status: merge_field("status", &l.status, &r.status)?,
            quality: merge_field("quality", &l.quality, &r.quality)?,
            release_group_type: merge_field("release_group_type", &l.release_group_type, &r.release_group_type)?,
            release_group_musicbrainz_id: merge_field(
                "release_group_musicbrainz_id",
                &l.release_group_musicbrainz_id,
                &r.release_group_musicbrainz_id
            )?,
            
            discogs_id: merge_field("discogs_id", &l.discogs_id, &r.discogs_id)?,
            lastfm_id: merge_field("lastfm_id", &l.lastfm_id, &r.lastfm_id)?,
            musicbrainz_id: merge_field("musicbrainz_id", &l.musicbrainz_id, &r.musicbrainz_id)?,
            spotify_id: merge_field("spotify_id", &l.spotify_id, &r.spotify_id)?,
            wikidata_id: merge_field("wikidata_id", &l.wikidata_id, &r.wikidata_id)?,
        })
    }
}

/// TODO okay moving forward with this cause it's better than what I have, but
/// want to note that I think I probably need to go ahead and add a "quality"
/// score either to the entity or even to every field. Then, when merging new
/// data, when there is a conflict, you choose the higher quality. 
/// I think this really only applies (currently) to merge_field_case_insensitive
/// since when there is a mix of cases it has to choose one or the other. 
impl MergeRules<Track> for Track {
    fn try_merge(l: &Track, r: &Track) -> Result<Track, MergeError> {
        Ok(Track {
            id: merge_field("id", &l.id, &r.id)?,
            title: merge_field_case_insensitive("title", &l.title, &r.title)?,
            disambiguation: merge_field("disambiguation", &l.disambiguation, &r.disambiguation)?,
            summary: merge_field("summary", &l.summary, &r.summary)?,
            save: l.save || r.save,
            download: l.download || r.download,
            
            release_id: merge_field("release_id", &l.release_id, &r.release_id)?,
            position: merge_field("position", &l.position, &r.position)?,
            length_ms: merge_field("length_ms", &l.length_ms, &r.length_ms)?,
            lyrics: merge_field("lyrics", &l.lyrics, &r.lyrics)?,
            synchronized_lyrics: merge_field("synchronized_lyrics", &l.synchronized_lyrics, &r.synchronized_lyrics)?,
            media_track_count: merge_field("media_track_count", &l.media_track_count, &r.media_track_count)?,
            media_position: merge_field("media_position", &l.media_position, &r.media_position)?,
            media_title: merge_field("media_title", &l.media_title, &r.media_title)?,
            media_format: merge_field("media_format", &l.media_format, &r.media_format)?,

            discogs_id: merge_field("discogs_id", &l.discogs_id, &r.discogs_id)?,
            lastfm_id: merge_field("lastfm_id", &l.lastfm_id, &r.lastfm_id)?,
            musicbrainz_id: merge_field("musicbrainz_id", &l.musicbrainz_id, &r.musicbrainz_id)?,
            spotify_id: merge_field("spotify_id", &l.spotify_id, &r.spotify_id)?,
            wikidata_id: merge_field("wikidata_id", &l.wikidata_id, &r.wikidata_id)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge_compatible_releases() {
        let release1 = Release {
            title: Some("Test Album".to_string()),
            date: Some("2020-01-01".to_string()),
            musicbrainz_id: Some("mbid-123".to_string()),
            ..Default::default()
        };

        let release2 = Release {
            title: Some("Test Album".to_string()),
            date: Some("2020-01-01".to_string()),
            musicbrainz_id: Some("mbid-123".to_string()),
            country: Some("US".to_string()),  // Additional data
            ..Default::default()
        };

        let merged = Release::try_merge(&release1, &release2).unwrap();
        assert_eq!(merged.title, Some("Test Album".to_string()));
        assert_eq!(merged.date, Some("2020-01-01".to_string()));
        assert_eq!(merged.musicbrainz_id, Some("mbid-123".to_string()));
        assert_eq!(merged.country, Some("US".to_string()));
    }

    #[test]
    fn test_merge_conflicting_musicbrainz_ids() {
        let release1 = Release {
            title: Some("Test Album".to_string()),
            musicbrainz_id: Some("mbid-123".to_string()),
            ..Default::default()
        };

        let release2 = Release {
            title: Some("Test Album".to_string()),
            musicbrainz_id: Some("mbid-456".to_string()),
            ..Default::default()
        };

        let result = Release::try_merge(&release1, &release2);
        assert!(result.is_err());
        match result.unwrap_err() {
            MergeError::ConflictingField { field, .. } => {
                assert_eq!(field, "musicbrainz_id");
            }
            _ => panic!("Expected ConflictingField error"),
        }
    }

    #[test]
    fn test_merge_conflicting_dates() {
        let release1 = Release {
            title: Some("Live Album".to_string()),
            date: Some("2020-01-01".to_string()),
            ..Default::default()
        };

        let release2 = Release {
            title: Some("Live Album".to_string()),
            date: Some("2021-06-15".to_string()),
            ..Default::default()
        };

        let result = Release::try_merge(&release1, &release2);
        assert!(result.is_err());
        match result.unwrap_err() {
            MergeError::ConflictingField { field, .. } => {
                assert_eq!(field, "date");
            }
            _ => panic!("Expected ConflictingField error"),
        }
    }

    #[test]
    fn test_merge_with_null_dates() {
        let release1 = Release {
            title: Some("Studio Album".to_string()),
            date: None,
            ..Default::default()
        };

        let release2 = Release {
            title: Some("Studio Album".to_string()),
            date: Some("2023-03-03".to_string()),
            ..Default::default()
        };

        let merged = Release::try_merge(&release1, &release2).unwrap();
        assert_eq!(merged.date, Some("2023-03-03".to_string()));
    }

    #[test]
    fn test_merge_case_insensitive_titles() {
        let release1 = Release {
            title: Some("Dark Side of the Moon".to_string()),
            ..Default::default()
        };

        let release2 = Release {
            title: Some("dark side of the moon".to_string()),
            barcode: Some("123456".to_string()),
            ..Default::default()
        };

        let merged = Release::try_merge(&release1, &release2).unwrap();
        assert_eq!(merged.title, Some("Dark Side of the Moon".to_string()));
        assert_eq!(merged.barcode, Some("123456".to_string()));
    }

    #[test]
    fn test_merge_conflicting_barcodes() {
        let release1 = Release {
            title: Some("Test Album".to_string()),
            barcode: Some("123456".to_string()),
            ..Default::default()
        };

        let release2 = Release {
            title: Some("Test Album".to_string()),
            barcode: Some("789012".to_string()),
            ..Default::default()
        };

        let result = Release::try_merge(&release1, &release2);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_saves_and_downloads() {
        let release1 = Release {
            title: Some("Test Album".to_string()),
            save: true,
            download: false,
            ..Default::default()
        };

        let release2 = Release {
            title: Some("Test Album".to_string()),
            save: false,
            download: true,
            ..Default::default()
        };

        let merged = Release::try_merge(&release1, &release2).unwrap();
        assert!(merged.save);  // OR operation
        assert!(merged.download);  // OR operation
    }
}
