use fractional_index::FractionalIndex;
use serde::{Deserialize, Serialize};
use crate::{library::Library, model::DimpleEntity};
use super::{PlaylistItem, Track};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Option<String>,
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

    pub source_id: Option<String>,
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
            JOIN Track ON (Track.id = PlaylistItem.Track_id)
            WHERE PlaylistItem.playlist_id = ?1
            AND PlaylistItem.deleted = FALSE
            ORDER BY PlaylistItem.ordinal ASC, PlaylistItem.rowid ASC
        ";
        library.query(sql, (self.id.clone(),))
    }

    pub fn items(&self, library: &Library) -> Vec<PlaylistItem> {
        let sql = "
            SELECT PlaylistItem.*
            FROM PlaylistItem
            WHERE PlaylistItem.playlist_id = ?1
            AND PlaylistItem.deleted = FALSE
            ORDER BY PlaylistItem.ordinal ASC, PlaylistItem.rowid ASC
        ";
        library.query(sql, (self.id.clone(),))
    }

    pub fn append(&self, library: &Library, model: &DimpleEntity) {
        self.insert(library, model, self.len(library));
    }

    // TODO I just had an idea for managing the positioning of albums in
    // the queue. When queueing something, just queue all the tracks but 
    // give each one a "grouping_id" and then when we encounter an item, 
    // we can treat items with the same grouping_id as equivalent.
    pub fn insert(&self, library: &Library, model: &DimpleEntity, index: usize) {
        match model {
            DimpleEntity::Artist(artist) => {
                for (i, release) in artist.releases(library).iter().enumerate() {
                    self.insert(library, &release.into(), index + i);
                }
            },
            DimpleEntity::Release(release) => {
                for (i, track) in release.tracks(library).iter().enumerate() {
                    self.insert(library, &track.into(), index + i);
                }
            },
            DimpleEntity::Track(track) => {
                let items = self.items(library);
                let index = index.min(items.len());
                let before = if index == 0 { 
                    None 
                } 
                else { 
                    items.get(index - 1).cloned().map(|b| b.ordinal) 
                };
                let after = items.get(index).cloned().map(|a| a.ordinal);
                let ordinal = Self::ordinal_between(&before, &after);
                let item = PlaylistItem {
                    id: None,
                    ordinal,
                    playlist_id: self.id.clone().unwrap(),
                    track_id: track.id.clone().unwrap(),
                    deleted: false,
                };
                let _item = library.save(&item);
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
    
    pub fn remove(&self, library: &Library, index: usize) {
        let items = self.items(library);
        if let Some(item) = items.get(index) {
            let mut updated_item = item.clone();
            updated_item.deleted = true;
            let _ = library.save(&updated_item);
        }
    }

    pub fn clear(&self, library: &Library) {
        let items = self.items(library);
        for mut item in items {
            item.deleted = true;
            let _ = library.save(&item);
        }
    }    
}

#[cfg(test)]
mod tests {
    use crate::{library::{self, Library}, model::{playlist::DimpleEntity, ModelBasics as _, Playlist, PlaylistItem, Release, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut model = library.save(&Playlist::default()).unwrap();
        assert!(model.id.is_some());
        assert!(model.name.is_none());
        model.name = Some("name".to_string());
        let model = library.save(&model).unwrap();
        let model: Playlist = library.get(&model.id.unwrap()).unwrap();
        assert!(model.name == Some("name".to_string()));
    }

    #[test]
    fn tracks() {
        let library = Library::open_memory();
        let playlist = library.save(&Playlist::default()).unwrap();
        for _ in 0..20 {
            let track = library.save(&Track::default()).unwrap();
            playlist.append(&library, &track.into());
        }
        let playlist = Playlist::get(&library, &playlist.id.unwrap()).unwrap();
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

    #[test]
    fn insert() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let playlist = Playlist::default().save(&library);
        let track1 = Track {
            title: Some("track1".to_string()),
            ..Default::default()
        }.save(&library);
        let track2 = Track {
            title: Some("track2".to_string()),
            ..Default::default()
        }.save(&library);
        let track3 = Track {
            title: Some("track3".to_string()),
            ..Default::default()
        }.save(&library);
        let track4 = Track {
            title: Some("track4".to_string()),
            ..Default::default()
        }.save(&library);
        let track5 = Track {
            title: Some("track5".to_string()),
            ..Default::default()
        }.save(&library);
        playlist.append(&library, &DimpleEntity::from(&track1));
        playlist.append(&library, &DimpleEntity::from(&track2));
        playlist.append(&library, &DimpleEntity::from(&track3));
        playlist.insert(&library, &track4.into(), 1);
        playlist.insert(&library, &track5.into(), 0);
        playlist.append(&library, &DimpleEntity::from(&track1));
        // TODO finish these tests
        // dbg!(PlaylistItem::list(&library));
        // dbg!(playlist.tracks(&library).iter().map(|t| t.title.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn insert2() {
        // it's play now on release page
        // so inserting a release with index = 1 (current_index = 0, + 1)
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let release = Release::default().save(&library);
        for i in 0..10 {
            Track {
                release_id: release.id.clone(),
                title: Some(format!("track {}", i)),
                ..Default::default()
            }.save(&library);
        }
        let playlist = Playlist::default().save(&library);
        playlist.insert(&library, &release.into(), 1);
        // TODO finish these tests
        // dbg!(PlaylistItem::list(&library));
        // dbg!(playlist.tracks(&library).iter().map(|t| t.title.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn test_remove() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let playlist = Playlist::default().save(&library);
        
        // Add 5 tracks to the playlist
        let mut tracks = Vec::new();
        for i in 0..5 {
            let track = Track {
                title: Some(format!("track{}", i)),
                ..Default::default()
            }.save(&library);
            tracks.push(track.clone());
            playlist.append(&library, &track.into());
        }
        
        // Verify all 5 tracks are present
        assert_eq!(playlist.len(&library), 5);
        
        // Remove the track at index 2 (track2)
        playlist.remove(&library, 2);
        
        // Verify we now have 4 tracks
        assert_eq!(playlist.len(&library), 4);
        
        // Verify the correct tracks remain
        let remaining_tracks = playlist.tracks(&library);
        assert_eq!(remaining_tracks[0].title, Some("track0".to_string()));
        assert_eq!(remaining_tracks[1].title, Some("track1".to_string()));
        assert_eq!(remaining_tracks[2].title, Some("track3".to_string()));
        assert_eq!(remaining_tracks[3].title, Some("track4".to_string()));
        
        // Remove first track
        playlist.remove(&library, 0);
        assert_eq!(playlist.len(&library), 3);
        
        // Remove last track
        playlist.remove(&library, 2);
        assert_eq!(playlist.len(&library), 2);
        
        let final_tracks = playlist.tracks(&library);
        assert_eq!(final_tracks[0].title, Some("track1".to_string()));
        assert_eq!(final_tracks[1].title, Some("track3".to_string()));
    }

    #[test]
    fn test_clear() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let playlist = Playlist::default().save(&library);
        
        // Add 10 tracks to the playlist
        for i in 0..10 {
            let track = Track {
                title: Some(format!("track{}", i)),
                ..Default::default()
            }.save(&library);
            playlist.append(&library, &track.into());
        }
        
        // Verify all 10 tracks are present
        assert_eq!(playlist.len(&library), 10);
        
        // Clear the playlist
        playlist.clear(&library);
        
        // Verify the playlist is now empty
        assert_eq!(playlist.len(&library), 0);
        assert_eq!(playlist.tracks(&library).len(), 0);
        assert_eq!(playlist.items(&library).len(), 0);
        
        // Add new tracks after clearing
        for i in 0..3 {
            let track = Track {
                title: Some(format!("new_track{}", i)),
                ..Default::default()
            }.save(&library);
            playlist.append(&library, &track.into());
        }
        
        // Verify new tracks are added correctly
        assert_eq!(playlist.len(&library), 3);
        let new_tracks = playlist.tracks(&library);
        assert_eq!(new_tracks[0].title, Some("new_track0".to_string()));
        assert_eq!(new_tracks[1].title, Some("new_track1".to_string()));
        assert_eq!(new_tracks[2].title, Some("new_track2".to_string()));
    }
}