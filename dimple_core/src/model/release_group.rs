use std::fmt;
use std::hash::Hash;

use dimple_db::db::transaction::DbTransaction;
use dimple_db::rusqlite::types::ToSqlOutput;
use dimple_db::rusqlite::ToSql;
use serde::Deserialize;
use serde::Serialize;

use crate::library::Library;
use crate::model::Artist;
use crate::model::Dimage;
use crate::model::Genre;
use crate::model::Link;
use crate::model::Release;

// https://musicbrainz.org/doc/ReleaseGroup
// https://musicbrainz.org/ws/2/release-group/1b4f4b3c-ca01-37b7-af1d-3e37989f86ad?inc=aliases%2Bartist-credits%2Breleases&fmt=json
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReleaseGroup {
    pub id: Option<String>,
    pub title: Option<String>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub save: bool,
    pub download: bool,

    pub first_release_date: Option<String>,
    pub primary_type: Option<ReleaseGroupPrimaryType>,

    pub discogs_id: Option<String>,
    pub lastfm_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub spotify_id: Option<String>,
    pub wikidata_id: Option<String>,
}

impl ReleaseGroup {
    pub fn artist(&self, library: &Library) -> Option<Artist> {
        self.artists(library).first().cloned()
    }

    pub fn artist_name(&self, library: &Library) -> Option<String> {
        self.artist(library).and_then(|a| a.name)
    }

    /// TODO this should return the artists in order, with the primary being
    /// first. I'm not exactly sure how to indicate primary yet.
    pub fn artists(&self, library: &Library) -> Vec<Artist> {
        library.query("
            SELECT a.* FROM ArtistRef ar 
            JOIN Artist a ON (a.id = ar.artist_id) 
            WHERE ar.model_id = ?1
            ORDER BY ar.rowid ASC
        ", (self.id.clone().unwrap(),))
    }

    pub fn secondary_types(&self, library: &Library) -> anyhow::Result<Vec<ReleaseGroupSecondaryType>> {
        let sql = "
            SELECT ReleaseGroupSecondaryTypeRef.* FROM ReleaseGroupSecondaryTypeRef
            WHERE ReleaseGroupSecondaryTypeRef.release_group_id = ?1
        ";
        let refs: Vec<ReleaseGroupSecondaryTypeRef> = library.query(sql, (self.id.clone(),));
        Ok(refs.iter().map(|r| r.secondary_type.clone()).collect::<Vec<_>>())
    }

    pub fn images(&self, library: &Library) -> Vec<Dimage> {
        library.query("
            SELECT d.* FROM DimageRef dr 
            JOIN Dimage d ON (d.id = dr.dimage_id) 
            WHERE dr.model_id = ?1
        ", (self.id.clone().unwrap(),))
    }

    pub fn links(&self, library: &Library) -> Vec<Link> {
        library.query("
            SELECT l.* FROM LinkRef lr 
            JOIN Link l ON (l.id = lr.link_id) 
            WHERE lr.model_id = ?1
        ", (self.id.clone().unwrap(),))
    }

    pub fn releases(&self, library: &Library) -> Vec<Release> {
        library.query("
            SELECT Release.* 
            FROM Release 
            WHERE Release.release_group_id = ?
            ORDER BY Release.date ASC NULLS LAST, Release.title ASC, Release.id ASC
        ", (self.id.clone().unwrap(),))
    }

    pub fn genres(&self, library: &Library) -> Vec<Genre> {
        library.query("
            SELECT g.* FROM GenreRef gr 
            JOIN Genre g ON (g.id = gr.genre_id) 
            WHERE gr.model_id = ?1
        ", (self.id.clone().unwrap(),))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseGroupPrimaryType {
    Album,
    Single,
    EP,
    Broadcast,
    Other,
    Unrecognized,
}

impl fmt::Display for ReleaseGroupPrimaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseGroupPrimaryType::Album => write!(f, "Album"),
            ReleaseGroupPrimaryType::Broadcast => write!(f, "Broadcast"),
            ReleaseGroupPrimaryType::EP => write!(f, "EP"),
            ReleaseGroupPrimaryType::Other => write!(f, "Other"),
            ReleaseGroupPrimaryType::Single => write!(f, "Single"),
            ReleaseGroupPrimaryType::Unrecognized => write!(f, "Unrecognized"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseGroupSecondaryType {
    Compilation,
    Soundtrack,
    Spokenword,
    Interview,
    Audiobook,
    AudioDrama,
    Live,
    Remix,
    DJMix,
    MixtapeStreet,
    Demo,
    FieldRecording,
    Unrecognized,
}


impl fmt::Display for ReleaseGroupSecondaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseGroupSecondaryType::AudioDrama => write!(f, "AudioDrama"),
            ReleaseGroupSecondaryType::Audiobook => write!(f, "Audiobook"),
            ReleaseGroupSecondaryType::Compilation => write!(f, "Compilation"),
            ReleaseGroupSecondaryType::DJMix => write!(f, "DJMix"),
            ReleaseGroupSecondaryType::Demo => write!(f, "Demo"),
            ReleaseGroupSecondaryType::FieldRecording => write!(f, "FieldRecording"),
            ReleaseGroupSecondaryType::Interview => write!(f, "Interview"),
            ReleaseGroupSecondaryType::Live => write!(f, "Live"),
            ReleaseGroupSecondaryType::MixtapeStreet => write!(f, "MixtapeStreet"),
            ReleaseGroupSecondaryType::Remix => write!(f, "Remix"),
            ReleaseGroupSecondaryType::Soundtrack => write!(f, "Soundtrack"),
            ReleaseGroupSecondaryType::Spokenword => write!(f, "Spokenword"),
            ReleaseGroupSecondaryType::Unrecognized => write!(f, "Unrecognized"),
        }
    }
}

impl ToSql for ReleaseGroupSecondaryType {
    fn to_sql(&self) -> dimple_db::rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseGroupSecondaryTypeRef {
    pub id: Option<String>,
    pub release_group_id: String,
    pub secondary_type: ReleaseGroupSecondaryType,
}

impl ReleaseGroupSecondaryTypeRef {
    pub fn attach(txn: &DbTransaction, release_group: &ReleaseGroup, secondary_type: ReleaseGroupSecondaryType) -> Result<(), anyhow::Error> {
        let sql = "SELECT * FROM ReleaseGroupSecondaryTypeRef WHERE release_group_id = ? and secondary_type = ?";
        if txn.query::<ReleaseGroupSecondaryTypeRef, _>(sql, (release_group.id.as_ref(), &secondary_type))?.is_empty() {
            let _ = txn.save(&ReleaseGroupSecondaryTypeRef {
                id: None,
                release_group_id: release_group.id.clone().unwrap(),
                secondary_type,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{library::Library, model::{release_group::ReleaseGroupPrimaryType, ReleaseGroup, ReleaseGroupSecondaryType, ReleaseGroupSecondaryTypeRef}};

    #[test]
    fn library_crud() -> Result<()> {
        let library = Library::open_memory();
        let _ = library.db.transaction(|txn| {
            let release_group = txn.save(&ReleaseGroup {
                title: Some("Swords Over Fish".to_string()),
                first_release_date: Some("2011".to_string()),
                primary_type: Some(ReleaseGroupPrimaryType::Album),
                ..Default::default()
            })?;
            let _ = ReleaseGroupSecondaryTypeRef::attach(txn, &release_group, ReleaseGroupSecondaryType::Live);
            let _ = ReleaseGroupSecondaryTypeRef::attach(txn, &release_group, ReleaseGroupSecondaryType::MixtapeStreet);
            Ok(())
        });
        let _ = library.save(&ReleaseGroup {
            title: Some("Timeless Chickens".to_string()),
            first_release_date: None,
            ..Default::default()
        })?;
        let rgs: Vec<ReleaseGroup> = library.list();
        assert_eq!(rgs[0].secondary_types(&library)?.len(), 2);
        assert_eq!(rgs[1].secondary_types(&library)?.len(), 0);
        Ok(())
    }
}

