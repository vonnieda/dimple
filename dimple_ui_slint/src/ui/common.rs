use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Playlist;
use dimple_core::model::Release;
use dimple_core::model::Track;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

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

