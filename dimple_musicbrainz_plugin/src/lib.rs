use std::iter;
use std::sync::Mutex;
use std::time::{Instant, Duration};

use anyhow::{Error, Result};
use dimple_core::model::{Artist, ArtistCredit, Entity, Genre, KnownIds, Medium, Model, Recording, ReleaseGroup, Track};
use dimple_librarian::plugin::{PluginSupport, NetworkMode, Plugin};
use musicbrainz_rs::entity::artist::ArtistSearchQuery;
use musicbrainz_rs::entity::artist::Artist as MBArtist;
use musicbrainz_rs::entity::release_group::ReleaseGroup as MBReleaseGroup;
use musicbrainz_rs::entity::relations::RelationContent;
use musicbrainz_rs::entity::release_group::ReleaseGroupSearchQuery;
use musicbrainz_rs::{Browse, Fetch, Search};
use serde_json::map;

#[derive(Debug)]
pub struct MusicBrainzPlugin {
    rate_limit_lock: Mutex<Instant>,
}

impl Default for MusicBrainzPlugin {
    fn default() -> Self {
        musicbrainz_rs::config::set_user_agent(dimple_librarian::plugin::USER_AGENT);
        Self {
            rate_limit_lock: Mutex::new(Instant::now()),
        }
    }
}

impl MusicBrainzPlugin {
    /// Blocks until at least one second has passed since the last request.
    fn enforce_rate_limit(&self) {
        let mut last_request_time = self.rate_limit_lock.lock().unwrap();

        if let Some(time_passed) = Instant::now().checked_duration_since(*last_request_time) {
            if time_passed < Duration::from_secs(1) {
                let sleep_duration = Duration::from_secs(1) - time_passed;
                std::thread::sleep(sleep_duration);
            }
        }

        // Update the last request time
        *last_request_time = Instant::now();
    }
}

impl Plugin for MusicBrainzPlugin {
    fn name(&self) -> String {
        "MusicBrainz".to_string()
    }

    fn get(&self, model: &Model, network_mode: NetworkMode) -> Result<Option<Model>> {
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }

        match model {
            Model::Artist(artist) => {
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
                let request_token = PluginSupport::start_request(self, 
                    &format!("https://musicbrainz.org/ws/2/artist/{}?inc=aliases%20release-groups%20releases%20release-group-rels%20release-rels&fmt=json", mbid));
                self.enforce_rate_limit();
                let result = MBArtist::fetch().id(&mbid)
                    .with_aliases().with_annotations().with_genres().with_rating()
                    .with_tags().with_release_groups().with_url_relations()
                    .execute()?;
                PluginSupport::end_request(request_token, None, None);
                let artist = Artist::from(ArtistConverter::from(result.clone()));
                Ok(Some(artist.model()))
            },

            Model::ReleaseGroup(r) => {
                let mbid = r.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid missing"))?;
                let request_token = PluginSupport::start_request(self, 
                    &format!("https://musicbrainz.org/ws/2/release-group/{}?inc=aliases%20artists%20releases%20release-group-rels%20release-rels%20url-rels&fmt=json", mbid));
                self.enforce_rate_limit();
                let result = MBReleaseGroup::fetch().id(&mbid)
                    .with_aliases().with_annotations().with_artists()
                    .with_genres().with_ratings().with_releases().with_tags()
                    .with_url_relations()
                    .execute()?;
                PluginSupport::end_request(request_token, None, None);
                let release_group = ReleaseGroup::from(ReleaseGroupConverter::from(result.clone()));
                Ok(Some(release_group.model()))
            },

            _ => Ok(None),
        }
    }

    fn list(
        &self,
        list_of: &Model,
        related_to: &Option<Model>,
        network_mode: NetworkMode,
    ) -> Result<Box<dyn Iterator<Item = Model>>> {
        // TODO handle paging
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }
        match (list_of, related_to) {
            (Model::ReleaseGroup(_), Some(Model::Artist(artist))) => {                
                let mbid = artist.known_ids.musicbrainz_id.clone().ok_or(Error::msg("mbid required"))?;

                let request_token = PluginSupport::start_request(self, 
                    &format!("https://musicbrainz.org/ws/2/release-group/TODO TODO{}?fmt=json", mbid));
                self.enforce_rate_limit();
                let iter = MBReleaseGroup::browse().by_artist(&mbid).limit(100)
                    .execute()?
                    .entities
                    .into_iter()
                    .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
                    .map(|src| src.model());
                PluginSupport::end_request(request_token, None, None);
                Ok(Box::new(iter))
            },
            _ => Err(Error::msg("Not implemented.")),
        }        
    }

    fn search(&self, query: &str, network_mode: dimple_librarian::plugin::NetworkMode) 
        -> Result<Box<dyn Iterator<Item = Model>>> {
        if network_mode != NetworkMode::Online {
            return Err(Error::msg("Offline."))
        }
    
        let iter = std::iter::empty();

        // https://musicbrainz.org/ws/2/artist/?query=metallica&fmt=json
        let search_query = ArtistSearchQuery::query_builder()
            .artist(&query)
            .build();
        let request_token = PluginSupport::start_request(self, 
            &format!("https://musicbrainz.org/ws/2/artist/?query={}&fmt=json", &query));
        self.enforce_rate_limit();
        let artists = MBArtist::search(search_query)
            .execute()?
            .entities
            .into_iter()
            .map(|result| Artist::from(ArtistConverter::from(result)))
            .map(|result| result.model());
        PluginSupport::end_request(request_token, None, None);
        let iter = iter.chain(artists);

        let search_query = ReleaseGroupSearchQuery::query_builder()
            .release_group(&query)
            .build();
        let request_token = PluginSupport::start_request(self, 
            &format!("https://musicbrainz.org/ws/2/release-group/?query={}&fmt=json", &query));
        self.enforce_rate_limit();
        let release_groups = MBReleaseGroup::search(search_query)
            .execute()?
            .entities
            .into_iter()
            .map(|result| Into::<ReleaseGroup>::into(ReleaseGroupConverter::from(result)))
            .map(|result| result.model());
        PluginSupport::end_request(request_token, None, None);
        let iter = iter.chain(release_groups);

        // let search_query = ReleaseSearchQuery::query_builder()
        //     .release(&query)
        //     .build();
        // let releases = MBRelease::search(search_query)
        //     .execute()?
        //     .entities
        //     .into_iter()
        //     .map(|result| Into::<Release>::into(ReleaseConverter::from(result)))
        //     .map(|release| Box::new(release) as Box::<dyn Entity>);
        Ok(Box::new(iter))
    }    
}

// Note that in the converters below ..Default should never be used. If a Default
// is temporarily needed it can be specified on the field itself, but not
// the entire struct. This is to help avoid skipping fields when new ones
// are added.

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    }
    else {
        Some(s)
    }
}

pub struct ArtistConverter(musicbrainz_rs::entity::artist::Artist);

impl From<musicbrainz_rs::entity::artist::Artist> for ArtistConverter {
    fn from(value: musicbrainz_rs::entity::artist::Artist) -> Self {
        ArtistConverter(value)
    }
}

impl From<ArtistConverter> for dimple_core::model::Artist {
    fn from(value: ArtistConverter) -> Self {
        dimple_core::model::Artist {
            country: value.0.country,
            disambiguation: none_if_empty(value.0.disambiguation),
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .collect(),
            name: none_if_empty(value.0.name),
            summary: None,
        }
    }
}

pub struct ArtistCreditConverter(musicbrainz_rs::entity::artist_credit::ArtistCredit);

impl From<musicbrainz_rs::entity::artist_credit::ArtistCredit> for ArtistCreditConverter {
    fn from(value: musicbrainz_rs::entity::artist_credit::ArtistCredit) -> Self {
        ArtistCreditConverter(value)
    }
}

impl From<ArtistCreditConverter> for dimple_core::model::ArtistCredit {
    fn from(value: ArtistCreditConverter) -> Self {
        ArtistCredit {
            artist: Artist::from(ArtistConverter::from(value.0.artist)),
            join_phrase: value.0.joinphrase,
            key: None,
            name: Some(value.0.name),
        }
    }
}

impl From<ArtistCreditConverter> for dimple_core::model::Artist {
    fn from(value: ArtistCreditConverter) -> Self {
        Artist::from(ArtistConverter::from(value.0.artist))
    }
}

pub struct ReleaseGroupConverter(musicbrainz_rs::entity::release_group::ReleaseGroup);

impl From<musicbrainz_rs::entity::release_group::ReleaseGroup> for ReleaseGroupConverter {
    fn from(value: musicbrainz_rs::entity::release_group::ReleaseGroup) -> Self {
        ReleaseGroupConverter(value)
    }
}

impl From<ReleaseGroupConverter> for dimple_core::model::ReleaseGroup {
    fn from(value: ReleaseGroupConverter) -> Self {
        dimple_core::model::ReleaseGroup {
            annotation: value.0.annotation,
            artist_credits: value.0.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            disambiguation: none_if_empty(value.0.disambiguation),
            first_release_date: value.0.first_release_date.map(|f| f.to_string()),
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .collect(),
            primary_type: value.0.primary_type.map(|f| format!("{:?}", f)),
            summary: None,
            title: none_if_empty(value.0.title),
        }
    }
}

pub struct ReleaseConverter(musicbrainz_rs::entity::release::Release);

impl From<musicbrainz_rs::entity::release::Release> for ReleaseConverter {
    fn from(value: musicbrainz_rs::entity::release::Release) -> Self {
        ReleaseConverter(value)
    }
}

impl From<ReleaseConverter> for dimple_core::model::Release {
    fn from(value: ReleaseConverter) -> Self {
        dimple_core::model::Release {
            artist_credits: value.0.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            barcode: value.0.barcode,
            country: value.0.country,
            date: value.0.date.map(|f| f.to_string()),
            disambiguation: value.0.disambiguation,
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .collect(),
            title: none_if_empty(value.0.title),
            packaging: value.0.packaging.map(|f| format!("{:?}", f)),
            primary_type: None,
            release_group: value.0.release_group
                .map(|f| ReleaseGroup::from(ReleaseGroupConverter::from(f.to_owned()))).unwrap(),
            status: value.0.status.map(|f| format!("{:?}", f)),
            summary: None,
            media: value.0.media.iter().flatten()
                .map(|f| Medium::from(MediumConverter::from(f.to_owned())))
                .collect(),
        }
    }
}

pub struct MediumConverter(musicbrainz_rs::entity::release::Media);

impl From<musicbrainz_rs::entity::release::Media> for MediumConverter {
    fn from(value: musicbrainz_rs::entity::release::Media) -> Self {
        MediumConverter(value)
    }
}

impl From<MediumConverter> for dimple_core::model::Medium {
    fn from(value: MediumConverter) -> Self {
        dimple_core::model::Medium {
            key: None,
            title: value.0.title,            
            disc_count: value.0.disc_count,
            format: value.0.format,
            position: value.0.position,
            track_count: Some(value.0.track_count),
            tracks: value.0.tracks.iter().flatten()
                .map(|f| Track::from(TrackConverter::from(f.to_owned())))
                .collect(),
        }
    }
}

pub struct TrackConverter(musicbrainz_rs::entity::release::Track);

impl From<musicbrainz_rs::entity::release::Track> for TrackConverter {
    fn from(value: musicbrainz_rs::entity::release::Track) -> Self {
        TrackConverter(value)
    }
}

impl From<TrackConverter> for dimple_core::model::Track {
    fn from(value: TrackConverter) -> Self {
        dimple_core::model::Track {
            artist_credits: value.0.recording.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            genres: value.0.recording.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            length: value.0.length,
            number: u32::from_str_radix(&value.0.number, 10).ok(),
            position: Some(value.0.position),
            recording: Recording::from(RecordingConverter::from(value.0.recording.to_owned())),
            title: none_if_empty(value.0.title),
        }
    }
}

pub struct RecordingConverter(musicbrainz_rs::entity::recording::Recording);

impl From<musicbrainz_rs::entity::recording::Recording> for RecordingConverter {
    fn from(value: musicbrainz_rs::entity::recording::Recording) -> Self {
        RecordingConverter(value)
    }
}

impl From<RecordingConverter> for dimple_core::model::Recording {
    fn from(value: RecordingConverter) -> Self {
        dimple_core::model::Recording {
            annotation: value.0.annotation,
            artist_credits: value.0.artist_credit.iter().flatten()
                .map(|artist_credit| ArtistCredit::from(ArtistCreditConverter::from(artist_credit.to_owned())))
                .collect(),
            disambiguation: value.0.disambiguation,
            genres: value.0.genres.iter().flatten()
                .map(|f| Genre::from(GenreConverter::from(f.to_owned())))
                .collect(),
            isrc: value.0.isrcs.into_iter().flatten().next(),
            key: None,
            known_ids: KnownIds {
                musicbrainz_id: Some(value.0.id),
                ..Default::default()
            },
            length: value.0.length,
            links: value.0.relations.iter().flatten()
                .filter_map(|r| match &r.content {
                    RelationContent::Url(u) => Some(u.resource.to_string()),
                    _ => None,
                })
                .collect(),
            summary: None,
            title: none_if_empty(value.0.title),
        }
    }
}

pub struct GenreConverter(musicbrainz_rs::entity::genre::Genre);

impl From<musicbrainz_rs::entity::genre::Genre> for GenreConverter {
    fn from(value: musicbrainz_rs::entity::genre::Genre) -> Self {
        GenreConverter(value)
    }
}

impl From<GenreConverter> for dimple_core::model::Genre {
    fn from(value: GenreConverter) -> Self {
        dimple_core::model::Genre {
            disambiguation: None,
            key: None,
            // known_ids: KnownIds {
            //     musicbrainz_id: Some(value.0.id),
            //     ..Default::default()
            // },
            // links: value.0.relations.iter().flatten()
            //     .filter_map(|r| match &r.content {
            //         RelationContent::Url(u) => Some(u.resource.to_string()),
            //         _ => None,
            //     })
            //     .collect(),
            name: none_if_empty(value.0.name),
            summary: None,
            ..Default::default()
        }
    }
}



// fn list(
//     &self,
//     list_of: &dyn Entity,
//     related_to: Option<&dyn Entity>,
//     network_mode: dimple_librarian::plugin::NetworkMode,
// ) -> Result<Box<dyn Iterator<Item = Box<dyn Entity>>>> {
//     match (list_of.model(), related_to.map(|related_to| model()) {
//         (Model::ReleaseGroup(_), Some(Entities::Artist(a))) => {                
//             // // TODO handle paging
//             let mbid = a.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/release-group/TODO TODO{}?fmt=json", mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBReleaseGroup::browse().by_artist(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                 .map(Entities::ReleaseGroup)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         (Entities::Release(_), Some(Entities::Artist(a))) => {                
//             // TODO handle paging
//             let mbid = a.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             // https://musicbrainz.org/artist/65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab/releases
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/release?artist={}&fmt=json", &mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBRelease::browse().by_artist(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                 .map(Entities::Release)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         (Entities::Recording(_), Some(Entities::Release(a))) => {
//             // TODO handle paging
//             let mbid = a.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/recording/TODO TODO{}?fmt=json", &mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBRecording::browse().by_release(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//                 .map(Entities::Recording)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         (Entities::Release(_), Some(Entities::ReleaseGroup(r))) => {
//             // TODO handle paging
//             let mbid = r.mbid();
//             if mbid.is_none() {
//                 return Box::new(vec![].into_iter())
//             }
//             let mbid = mbid.unwrap();
//             let request_token = LibrarySupport::start_request(self, 
//                 &format!("https://musicbrainz.org/ws/2/release/TODO TODO{}?fmt=json", &mbid));
//             self.enforce_rate_limit();
//             let results: Vec<_> = MBRelease::browse().by_release_group(&mbid)
//                 .execute()
//                 .inspect(|_f| {
//                     LibrarySupport::end_request(request_token, None, None);
//                 })        
//                 .inspect_err(|f| log::error!("{}", f))
//                 .unwrap()
//                 .entities
//                 .iter()
//                 .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                 .map(Entities::Release)
//                 .collect();
//             Box::new(results.into_iter())
//         },
//         _ => Box::new(vec![].into_iter()),
//     }
// }
    


// impl Collection for MusicBrainzLibrary {
//     fn name(&self) -> String {
//         "MusicBrainz".to_string()
//     }

//     fn search(&self, query: &str) -> Box<dyn Iterator<Item = Entities>> {
//         let query = query.to_string();

//         self.enforce_rate_limit();
//         let request_token = LibrarySupport::start_request(self, 
//             &format!("http://musicbrainz.org/search/artist/{}", &query));

//         // TODO And releases, tracks, etc.
//         let search_query = ArtistSearchQuery::query_builder()
//             .artist(&query)
//             .build();
//         let results: Vec<Entities> = MBArtist::search(search_query)
//             .execute()
//             .inspect(|_f| {
//                 LibrarySupport::end_request(request_token, None, None);
//             })
//             // TODO
//             .unwrap()
//             .entities
//             .iter()
//             .map(|src| dimple_core::model::Artist::from(ArtistConverter::from(src.clone())))
//             .map(Entities::Artist)
//             .collect();
//         Box::new(results.into_iter())
//     }

//     fn list(&self, of_type: &Entities, related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
//         match (of_type, related_to) {
//             (Entities::ReleaseGroup(_), Some(Entities::Artist(a))) => {                
//                 // // TODO handle paging
//                 let mbid = a.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release-group/TODO TODO{}?fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBReleaseGroup::browse().by_artist(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                     .map(Entities::ReleaseGroup)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             (Entities::Release(_), Some(Entities::Artist(a))) => {                
//                 // TODO handle paging
//                 let mbid = a.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 // https://musicbrainz.org/artist/65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab/releases
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release?artist={}&fmt=json", &mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBRelease::browse().by_artist(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(Entities::Release)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             (Entities::Recording(_), Some(Entities::Release(a))) => {
//                 // TODO handle paging
//                 let mbid = a.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/recording/TODO TODO{}?fmt=json", &mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBRecording::browse().by_release(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//                     .map(Entities::Recording)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             (Entities::Release(_), Some(Entities::ReleaseGroup(r))) => {
//                 // TODO handle paging
//                 let mbid = r.mbid();
//                 if mbid.is_none() {
//                     return Box::new(vec![].into_iter())
//                 }
//                 let mbid = mbid.unwrap();
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release/TODO TODO{}?fmt=json", &mbid));
//                 self.enforce_rate_limit();
//                 let results: Vec<_> = MBRelease::browse().by_release_group(&mbid)
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .unwrap()
//                     .entities
//                     .iter()
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(Entities::Release)
//                     .collect();
//                 Box::new(results.into_iter())
//             },
//             _ => Box::new(vec![].into_iter()),
//         }
//     }

//     fn fetch(&self, entity: &Entities) -> Option<Entities> {
//         match entity {
//             Entities::Artist(a) => {
//                 let mbid = a.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/artist/{}?inc=aliases%20release-groups%20releases%20release-group-rels%20release-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBArtist::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_genres().with_rating()
//                     .with_tags().with_release_groups().with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| Artist::from(ArtistConverter::from(src.clone())))
//                     .map(Entities::Artist)        
//             },
//             Entities::ReleaseGroup(r) => {
//                 let mbid = r.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release-group/{}?inc=aliases%20artists%20releases%20release-group-rels%20release-rels%20url-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBReleaseGroup::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_artists()
//                     .with_genres().with_ratings().with_releases().with_tags()
//                     .with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| ReleaseGroup::from(ReleaseGroupConverter::from(src.clone())))
//                     .map(Entities::ReleaseGroup)        
//             },
//             Entities::Release(r) => {
//                 let mbid = r.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/release/{}?inc=aliases%20artist-credits%20artist-rels%20artists%20genres%20labels%20ratings%20recording-rels%20recordings%20release-groups%20release-group-rels%20tags%20release-rels%20url-rels%20work-level-rels%20work-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBRelease::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_artist_credits()
//                     .with_artists().with_genres().with_labels().with_ratings()
//                     .with_recordings().with_release_groups().with_tags()
//                     .with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| Release::from(ReleaseConverter::from(src.clone())))
//                     .map(Entities::Release)        
//             },
//             Entities::Recording(r) => {
//                 let mbid = r.mbid()?;
//                 let request_token = LibrarySupport::start_request(self, 
//                     &format!("https://musicbrainz.org/ws/2/recording/{}?inc=aliases%20artist-credits%20artist-rels%20artists%20genres%20labels%20ratings%20recording-rels%20recordings%20release-groups%20release-group-rels%20tags%20release-rels%20url-rels%20work-level-rels%20work-rels&fmt=json", mbid));
//                 self.enforce_rate_limit();
//                 MBRecording::fetch().id(&mbid)
//                     .with_aliases().with_annotations().with_artists()
//                     .with_genres().with_isrcs().with_ratings().with_releases()
//                     .with_tags().with_url_relations()
//                     .execute()
//                     .inspect(|_f| {
//                         LibrarySupport::end_request(request_token, None, None);
//                     })        
//                     .inspect_err(|f| log::error!("{}", f))
//                     .ok()
//                     .inspect(|src| log::debug!("{:?}", src))
//                     .map(|src| Recording::from(RecordingConverter::from(src.clone())))
//                     .map(Entities::Recording)        
//             },
//             Entities::Genre(_) => None,
//             Entities::RecordingSource(_) => None,
//             Entities::MediaFile(_) => None,
//             Entities::Track(_) => None,
//             Entities::Medium(_) => None,
//         }        
//     }
// }