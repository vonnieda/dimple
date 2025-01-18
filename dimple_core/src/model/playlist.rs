use dimple_core_macro::ModelSupport;
use fractional_index::FractionalIndex;
use uuid::Uuid;
use crate::library::Library;
use crate::model::ModelBasics as _;

use super::{Artist, ModelBasics as _, PlaylistItem, Release, Track};

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,
}

impl Playlist {
    pub fn len(&self, library: &Library) -> usize {
        // TODO Change to select count()
        self.tracks(library).len()
    }

    pub fn tracks(&self, library: &Library) -> Vec<Track> {
        let sql = "
            SELECT Track.*
            FROM PlaylistItem
            JOIN Track ON (Track.key = PlaylistItem.Track_key)
            WHERE PlaylistItem.playlist_key = ?1
            ORDER BY PlaylistItem.ordinal ASC, PlaylistItem.rowid ASC
        ";
        library.query(sql, (self.key.clone(),))
    }

    pub fn append(&self, library: &Library, model: &impl Model) {
        self.insert(library, model, self.len(library));
    }

    pub fn insert(&self, library: &Library, model: &impl Model, index: usize) {
        log::info!("insert {} {:?} {} {}", 
            model.type_name(), 
            model.key(), 
            index, 
            self.len(library));
        match model.type_name().as_str() {
            "Artist" => {
                let artist = Artist::get(library, &model.key().unwrap()).unwrap();
                for (i, release) in artist.releases(library).iter().enumerate() {
                    self.insert(library, release, index + i);
                }
            },
            "Release" => {
                let release = Release::get(library, &model.key().unwrap()).unwrap();
                for (i, track) in release.tracks(library).iter().enumerate() {
                    self.insert(library, track, index + i);
                }
            },
            "Track" => {
                let track = Track::get(library, &model.key().unwrap()).unwrap();
                let items = PlaylistItem::query(library, "
                    SELECT PlaylistItem.* 
                    FROM PlaylistItem 
                    WHERE playlist_key = ? 
                    ORDER BY ordinal, rowid
                    LIMIT 2 OFFSET ?
                ", (&self.key, index.checked_sub(1).unwrap_or(0)));
                let before = items.get(0).cloned().map(|b| b.ordinal);
                let after = items.get(1).cloned().map(|a| a.ordinal);
                let ordinal = Self::ordinal_between(&before, &after);
                let _item = library.save(&PlaylistItem {
                    key: None,
                    ordinal,
                    playlist_key: self.key.clone().unwrap(),
                    track_key: track.key.clone().unwrap(),
                });
            },
            _ => todo!(),
        }
    }

    pub fn ordinal_between(left: &Option<String>, right: &Option<String>) -> String {
        match (left, right) {
            (None, None) => FractionalIndex::default().to_string(),
            (Some(left), None) => {
                let left = FractionalIndex::from_string(left).unwrap();
                FractionalIndex::new_after(&left).to_string()
            },
            (None, Some(right)) => {
                let right = FractionalIndex::from_string(right).unwrap();
                FractionalIndex::new_before(&right).to_string()
            }
            (Some(left), Some(right)) => {
                let left = FractionalIndex::from_string(left).unwrap();
                let right = FractionalIndex::from_string(right).unwrap();
                FractionalIndex::new_between(&left, &right).unwrap_or(left).to_string()
            }
        }
    }
    
    pub fn remove(&self, index: usize) {
        // TODO ordinals
    }

    pub fn clear(&self, library: &Library) {
        library.conn().execute("DELETE FROM PlaylistItem
            WHERE playlist_key = ?1", (self.key.clone().unwrap(),)).unwrap();
    }    
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, Model, ModelBasics as _, Playlist, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Playlist::default());
        assert!(model.key.is_some());
        assert!(model.name.is_none());
        model.name = Some("name".to_string());
        let model = library.save(&model);
        let model: Playlist = library.get(&model.key.unwrap()).unwrap();
        assert!(model.name == Some("name".to_string()));
    }

    #[test]
    fn tracks() {
        let library = Library::open_memory();
        let playlist = library.save(&Playlist::default());
        for _ in 0..20 {
            let track = library.save(&Track::default());
            playlist.append(&library, &track);
        }
        let playlist = Playlist::get(&library, &playlist.key.unwrap()).unwrap();
        assert!(playlist.len(&library) == 20);
    }

    #[test]
    fn ordinals() {
        let a = Playlist::ordinal_between(&None, &None);
        let b = Playlist::ordinal_between(&Some(a.clone()), &None);
        let c = Playlist::ordinal_between(&Some(a.clone()), &Some(b.clone()));
        assert!(a < b);
        assert!(a < c);
        assert!(c < b);
    }
}