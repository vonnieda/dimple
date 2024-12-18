use dimple_core_macro::ModelSupport;
use crate::library::Library;

use super::Track;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct Playlist {
    pub key: Option<String>,
    pub name: Option<String>,

    pub save: bool,
    pub download: bool,
    pub summary: Option<String>,
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
        ";
        library.query(sql, (self.key.clone(),))
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Diff, Model, Playlist, Track}};

    #[test]
    fn library_crud() {
        let library = Library::open("file:e3b2df54-d10a-4530-b753-1fc82295ad32?mode=memory&cache=shared");
        let mut model = library.save(&Playlist::default());
        assert!(model.key.is_some());
        assert!(model.name.is_none());
        model.name = Some("name".to_string());
        let model = library.save(&model);
        let model: Playlist = library.get(&model.key.unwrap()).unwrap();
        assert!(model.name == Some("name".to_string()));
    }

    #[test]
    fn migration_02() {
        let library = Library::open("file:f165f2a4-3b21-4053-86ea-259aad53825a?mode=memory&cache=shared");
        let model = library.save(&Playlist {
            save: true,
            ..Default::default()   
        });
        assert!(model.save == true);
    }

    #[test]
    fn diff() {
        let a = Playlist::default();
        let b = Playlist {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            ..Default::default()
        };
        let diff = a.diff(&b);
        assert!(diff.len() == 2);
        assert!(diff[0].field == Some("key".to_string()));
        assert!(diff[1].field == Some("name".to_string()));
    }

    #[test]
    fn apply_diff() {
        let a = Playlist::default();
        let b = Playlist {
            key: Some("key".to_string()),
            name: Some("name".to_string()),
            ..Default::default()
        };
        let diff = a.diff(&b);
        let mut c = Playlist::default();
        c.apply_diff(&diff);
        assert!(c == b);
    }

    #[test]
    fn tracks() {
        let library = Library::open("file:5d29e64a-5418-4773-b557-56448ae09efd?mode=memory&cache=shared");
        let playlist = library.save(&Playlist::default());
        for _ in 0..3 {
            let track = library.save(&Track::default());
            library.playlist_add(&playlist, &track.key.unwrap());
        }
        let playlist = library.get::<Playlist>(&playlist.key.unwrap()).unwrap();
        assert!(playlist.len(&library) == 3);
    }
}