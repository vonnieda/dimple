use std::collections::HashSet;

use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::KnownId;
use dimple_core::model::Medium;
use dimple_core::model::Picture;
use dimple_core::model::Playlist;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

use super::images::gen_fuzzy_circles;
use super::images::gen_fuzzy_rects;

impl From<Artist> for CardAdapter {
    fn from(value: Artist) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: value.disambiguation.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}

impl From<ReleaseGroup> for CardAdapter {
    fn from(value: ReleaseGroup) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://release-group/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://release-group/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: format!("{} {}", value.first_release_date.unwrap_or_default(), value.primary_type.unwrap_or_default()).into(),
                url: format!("dimple://release-group/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}

impl From<Release> for CardAdapter {
    fn from(value: Release) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: format!("{} {}", value.date.unwrap_or_default(), value.country.unwrap_or_default()).into(),
                url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}

impl From<Genre> for CardAdapter {
    fn from(value: Genre) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://genre/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://genre/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: value.disambiguation.unwrap_or_default().into(),
                url: format!("dimple://genre/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}

impl From<Playlist> for CardAdapter {
    fn from(value: Playlist) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            ..Default::default()
            // sub_title: LinkAdapter {
            //     name: value.disambiguation.unwrap_or_default().into(),
            //     url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
            // },
        }
    }
}

impl From<Track> for CardAdapter {
    fn from(value: Track) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://track/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://track/{}", value.key.clone().unwrap_or_default()).into(),
            },
            ..Default::default()
            // sub_title: LinkAdapter {
            //     name: value.disambiguation.unwrap_or_default().into(),
            //     url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
            // },
        }
    }
}


// fn recording_card(recording: &Recording, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Recording(recording.clone()), width, height),
//             link: Link {
//                 name: recording.title.str(),
//                 url: format!("dimple://recording/{}", recording.key.str()),
//             },
//         },
//         title: Link {
//             name: recording.title.str(),
//             url: format!("dimple://release/{}", recording.key.str()),
//         },
//         // TODO
//         // sub_title: 
//         ..Default::default()
//     }
// }

// fn artist_links(artist: &Artist) -> Vec<Link> {
//     artist.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
//             url: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
//         }))
//         .collect()
// }

// fn release_group_links(release_group: &ReleaseGroup) -> Vec<Link> {
//     release_group.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
//             url: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
//         }))
//         .collect()
// }

// fn release_links(release: &Release) -> Vec<Link> {
//     release.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
//             url: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
//         }))
//         .collect()
// }

// fn recording_links(recording: &Recording) -> Vec<Link> {
//     recording.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
//             url: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
//         }))
//         .collect()
// }

// fn card_adapters(cards: Vec<Card>) -> ModelRc<CardAdapter> {
//     let card_models: Vec<_>  = cards.iter()
//         .map(card_adapter)
//         .collect();
//     ModelRc::from(card_models.as_slice())
// }

// fn recording_adapters(recordings: Vec<Recording>) -> ModelRc<TrackAdapter> {
//     let adapters: Vec<_> = recordings.iter()
//         .map(|r| TrackAdapter {
//             title: LinkAdapter {
//                 name: r.title.str(),
//                 url: format!("dimple://recording/{}", r.key.str()).into(),
//             },
//             // track_number: t.number.clone().into(),
//             // length: length_to_string(t.length).into(),
//             artists: Default::default(),
//             plays: 0,
//             ..Default::default()
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// fn track_adapters(tracks: Vec<Track>) -> ModelRc<TrackAdapter> {
//     let adapters: Vec<_> = tracks.iter()
//         .map(|t| TrackAdapter {
//             title: LinkAdapter {
//                 name: t.title.clone().into(),
//                 url: format!("dimple://recording/{}", t.recording.key.str()).into(),
//             },
//             track_number: t.number.clone().into(),
//             length: length_to_string(t.length).into(),
//             artists: Default::default(),
//             plays: 0,
//             source_count: t.sources.len() as i32,
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// fn media_adapters(media: Vec<Medium>) -> ModelRc<MediumAdapter> {
//     let adapters: Vec<_> = media.iter()
//         .map(|m| MediumAdapter {
//             title: format!("{} {} of {}", m.format, m.position, m.disc_count).into(),
//             tracks: track_adapters(m.tracks.clone()),
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// Creates a simple score for a release to use when selecting a
// a default release.
// TODO this is super naive, just needed something to set the example.
// fn score_release(r: &Release) -> f64 {
//     let mut score = 0.;
//     let country = r.country.str().to_lowercase();
//     if country == "xw" {
//         score += 1.0;
//     }                
//     else if country == "us" || country == "gb" || country == "xe" {
//         score += 0.7;
//     }
//     else if !country.is_empty() {
//         score += 0.1;
//     }

//     if r.status.str().to_lowercase() == "official" {
//         score += 1.0;
//     }

//     let packaging = r.packaging.str().to_lowercase();
//     if packaging == "digipak" {
//         score += 1.0;
//     }
//     else if packaging == "jewelcase" {
//         score += 0.5;
//     }

//     // if !r.media.is_empty() {
//     //     let mut media_format_score = 0.;
//     //     for media in r.media.clone() {
//     //         let format = media.format.to_lowercase();
//     //         if format == "digital media" {
//     //             media_format_score += 1.0;
//     //         }
//     //         else if format == "cd" {
//     //             media_format_score += 0.5;
//     //         }
//     //     }
//     //     score += media_format_score / r.media.len() as f64;
//     // }

//     score / 4.
// }

pub fn create_links(count: usize) -> HashSet<String> {
    let mut links = HashSet::new();
    for i in 0..count {
        links.insert(format!("https://{}/{}/{}", 
            fakeit::internet::domain_name(), 
            fakeit::words::word(),
            fakeit::unique::uuid_v4()));
    }
    links
}

pub fn create_known_ids() -> HashSet<KnownId> {
    let mut known_ids = HashSet::new();
    known_ids.insert(KnownId::MusicBrainzId(fakeit::unique::uuid_v4()));
    known_ids
}

pub fn create_artist(db: &dyn Db) -> Artist {
    let artist = db.insert(&Artist {
        name: Some(fakeit::name::full()),
        summary: Some(fakeit::hipster::paragraph(1, 4, 40, " ".to_string())),
        country: Some(fakeit::address::country_abr()),
        disambiguation: Some(fakeit::address::country()),
        links: create_links(fakeit::misc::random(0, 5)),
        known_ids: create_known_ids(),
        ..Default::default()
    }.model()).unwrap();
    
    let artist_pic = db.insert(&Picture::new(&gen_fuzzy_circles(1000, 1000)).model()).unwrap();
    db.link(&artist_pic, &artist).unwrap();

    for _ in 0..3 {
        let genre = db.insert(&Genre {
            name: Some(format!("{} {}", fakeit::hipster::word(), 
                fakeit::words::word())),
            ..Default::default()
        }.model()).unwrap();
        let genre_pic = db.insert(&Picture::new(&gen_fuzzy_rects(1000, 1000)).model()).unwrap();
        db.link(&genre_pic, &genre).unwrap();
        db.link(&genre, &artist).unwrap();
    }
    
    artist.into()
}

pub fn create_release_group(db: &dyn Db) -> ReleaseGroup {
    let primary_type_options = vec!["album", "single", "ep", "other"];
    let release_group = db.insert(&ReleaseGroup {
        title: Some(fakeit::hipster::sentence(2)),
        summary: Some(fakeit::hipster::paragraph(2, 2, 40, " ".to_string())),
        disambiguation: None,
        first_release_date: Some(fakeit::datetime::year()),
        primary_type: Some(primary_type_options.get(fakeit::misc::random(0, primary_type_options.len() - 1)).unwrap().to_string()),
        links: create_links(fakeit::misc::random(0, 5)),
        known_ids: create_known_ids(),
        ..Default::default()
    }.model()).unwrap();
    
    let release_group_pic = db.insert(&Picture::new(&gen_fuzzy_rects(1000, 1000)).model()).unwrap();
    db.link(&release_group_pic, &release_group).unwrap();

    for _ in 0..3 {
        let genre = db.insert(&Genre {
            name: Some(format!("{} {}", fakeit::hipster::word(), 
                fakeit::words::word())),
            ..Default::default()
        }.model()).unwrap();
        let genre_pic = db.insert(&Picture::new(&gen_fuzzy_rects(1000, 1000)).model()).unwrap();
        db.link(&genre_pic, &genre).unwrap();
        db.link(&genre, &release_group).unwrap();
    }

    release_group.into()
}

pub fn create_release(db: &dyn Db) -> Release {
    let status_options = vec!["official", "promotion", "bootleg", "pseudo-release", "withdrawn", "cancelled"];
    let packaging_options = vec!["Book", "Box", "Digipak", "Jewel case", "Other", "Cardboard/Paper Sleeve"];
    let release = db.insert(&Release {
        title: Some(fakeit::hipster::sentence(2)),
        summary: Some(fakeit::hipster::paragraph(2, 2, 40, " ".to_string())),
        disambiguation: None,
        links: create_links(fakeit::misc::random(0, 5)),
        known_ids: create_known_ids(),
        barcode: Some(format!("{}{}", fakeit::address::zip(), fakeit::address::zip())),
        country: Some(fakeit::address::country_abr()),
        status: Some(status_options.get(fakeit::misc::random(0, status_options.len() - 1)).unwrap().to_string()),
        packaging: Some(packaging_options.get(fakeit::misc::random(0, packaging_options.len() - 1)).unwrap().to_string()),
        date: Some(fakeit::datetime::year()),        
        ..Default::default()
    }.model()).unwrap();
    
    let release_pic = db.insert(&Picture::new(&gen_fuzzy_rects(1000, 1000)).model()).unwrap();
    db.link(&release_pic, &release).unwrap();

    for _ in 0..3 {
        let genre = db.insert(&Genre {
            name: Some(format!("{} {}", fakeit::hipster::word(), 
                fakeit::words::word())),
            ..Default::default()
        }.model()).unwrap();
        db.link(&genre, &release).unwrap();
        let genre_pic = db.insert(&Picture::new(&gen_fuzzy_rects(1000, 1000)).model()).unwrap();
        db.link(&genre_pic, &genre).unwrap();
    }

    release.into()
}

pub fn create_medium(db: &dyn Db) -> Medium {
    let medium = db.insert(&Medium {
        ..Default::default()
    }.model()).unwrap();
    
    medium.into()
}

pub fn create_track(db: &dyn Db) -> Track {
    let track = db.insert(&Track {
        title: Some(fakeit::hipster::sentence(5)),
        length: Some(fakeit::misc::random(1, 30 * 60)),
        position: Some(fakeit::misc::random(1, 12)),        
        ..Default::default()
    }.model()).unwrap();
    
    track.into()
}

pub fn create_playlist(db: &dyn Db) -> Playlist {
    let playlist = db.insert(&Playlist {
        name: Some(fakeit::hipster::sentence(4)),
        ..Default::default()
    }.model()).unwrap();
    
    playlist.into()
}

pub fn create_random_data(db: &dyn Db, num_artists: u32) {
    for _ in 0..5 {
        let _ = create_playlist(db);
    }
    (0..num_artists).into_par_iter().for_each(|_| {
        let artist = create_artist(db);
        (0..fakeit::misc::random(0, 7)).into_par_iter().for_each(|_| {
            let release_group = create_release_group(db);
            db.link(&release_group.model(), &artist.model()).unwrap();            
            (0..fakeit::misc::random(1, 3)).into_par_iter().for_each(|_| {
                let release = create_release(db);
                db.link(&release.model(), &release_group.model()).unwrap();            
                (0..fakeit::misc::random(1, 2)).into_par_iter().for_each(|_| {
                    let medium = create_medium(db);
                    db.link(&medium.model(), &release.model()).unwrap();            
                    (0..fakeit::misc::random(1, 12)).into_par_iter().for_each(|_| {
                        let track = create_track(db);
                        db.link(&track.model(), &medium.model()).unwrap();            
                    });
                });
            });
        });
    });
}
