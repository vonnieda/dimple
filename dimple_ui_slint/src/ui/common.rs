use std::collections::HashSet;

use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::KnownIds;
use dimple_core::model::Medium;
use dimple_core::model::Dimage;
use dimple_core::model::Playlist;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use slint::ModelRc;
use crate::ui::CardAdapter;
use crate::ui::TrackAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

use super::image_gen::gen_fuzzy_circles;
use super::image_gen::gen_fuzzy_rects;

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
                name: value.disambiguation.clone().unwrap_or("Artist".to_string()).into(),
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
//             // length: length_to_string(t.length).into(),
//             // artists: Default::default(),
//             // plays: 0,
//             // source_count: t.sources.len() as i32,
//             ..Default::default()
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
