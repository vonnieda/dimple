use dimple_core::model::Genre;
use dimple_core::model::Model;
use dimple_core::model::ReleaseGroup;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;


// TODO Think this is a general problem with the lazy load, in that it can
// still be running after the page has changed. Needs to be cancellable, either
// way, but also should lfail more gracefully.
// 
// thread 'main' panicked at dimple_ui_slint/src/ui/pages/search.rs:48:78:
// called `Option::unwrap()` on a `None` value
// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
// fatal runtime error: failed to initiate panic, error 5
// Abort trap: 6


pub fn search(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();

    /// So this will be the first new model controller with the intention of
    /// being reactive. The goal will be to get an iterator from the search
    /// and feed those objects in realtime over to the UI as they come in.
    /// This will require adding a sort model, and figuring out that stuff
    /// so that the results stay sorted in the UI.

    std::thread::spawn(move || {
        log::info!("{}", url);
        let url = Url::parse(&url).unwrap();
        let query = url.path_segments().unwrap().next().unwrap();
        // TODO wtf? really?
        let query = percent_encoding::percent_decode_str(query).decode_utf8_lossy().to_string();

        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Navigator>().set_busy(true);
        }).unwrap();
        let results: Vec<Model> = librarian
            .search(&query)
            .unwrap()
            .collect();

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = results.iter().cloned().enumerate()
                .map(|(index, result)| {
                    let mut card: CardAdapter = model_card(&result);
                    card.image.image = images.lazy_get(result, 200, 200, move |ui, image| {
                        let mut card = ui.get_search().cards.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_search().cards.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_search(adapter);
            ui.set_page(Page::Search);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}


fn model_card(model: &Model) -> CardAdapter {
    match model {
        Model::Artist(artist) => artist.clone().into(),
        Model::ReleaseGroup(release_group) => release_group_card(release_group),
        Model::Genre(genre) => genre_card(genre),
        Model::Track(track) => track.clone().into(),
        _ => todo!(),
    }
}

pub fn card(title: &str, title_url: &str, 
    sub_title: &str, sub_title_url: &str,
    image_name: &str, image_url: &str) -> CardAdapter {
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: image_name.into(),
            url: image_url.into(),
        },
        title: LinkAdapter {
            name: title.into(),
            url: title_url.into(),
        },
        sub_title: LinkAdapter {
            name: sub_title.into(),
            url: sub_title_url.into(),
        },
    }    
}

pub fn release_group_card(release_group: &ReleaseGroup) -> CardAdapter {
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release_group.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release-group/{}", release_group.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: release_group.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release-group/{}", release_group.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: format!("{} {}", 
                release_group.first_release_date.clone().map(|date| date[..4].to_string()).unwrap_or_default(), 
                release_group.primary_type.clone().unwrap_or_default()).into(),
            url: format!("dimple://release-group/{}", release_group.key.clone().unwrap_or_default()).into(),
        },
    }    
}

pub fn genre_card(genre: &Genre) -> CardAdapter {
    let genre = genre.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: "Genre".into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
    }
}

// impl From<Artist> for CardAdapter {
//     fn from(value: Artist) -> Self {
//         CardAdapter {
//             image: ImageLinkAdapter {
//                 image: Default::default(),
//                 name: value.name.clone().unwrap_or_default().into(),
//                 url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             title: LinkAdapter {
//                 name: value.name.clone().unwrap_or_default().into(),
//                 url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             sub_title: LinkAdapter {
//                 name: value.disambiguation.clone().unwrap_or_default().into(),
//                 url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//         }
//     }
// }

// impl From<ReleaseGroup> for CardAdapter {
//     fn from(value: ReleaseGroup) -> Self {
//         CardAdapter {
//             image: ImageLinkAdapter {
//                 image: Default::default(),
//                 name: value.title.clone().unwrap_or_default().into(),
//                 url: format!("dimple://release-group/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             title: LinkAdapter {
//                 name: value.title.clone().unwrap_or_default().into(),
//                 url: format!("dimple://release-group/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             sub_title: LinkAdapter {
//                 name: format!("{} {}", value.first_release_date.unwrap_or_default(), value.primary_type.unwrap_or_default()).into(),
//                 url: format!("dimple://release-group/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//         }
//     }
// }

// impl From<Release> for CardAdapter {
//     fn from(value: Release) -> Self {
//         CardAdapter {
//             image: ImageLinkAdapter {
//                 image: Default::default(),
//                 name: value.title.clone().unwrap_or_default().into(),
//                 url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             title: LinkAdapter {
//                 name: value.title.clone().unwrap_or_default().into(),
//                 url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             sub_title: LinkAdapter {
//                 name: format!("{} {}", value.date.unwrap_or_default(), value.country.unwrap_or_default()).into(),
//                 url: format!("dimple://release/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//         }
//     }
// }

// impl From<Genre> for CardAdapter {
//     fn from(value: Genre) -> Self {
//         CardAdapter {
//             image: ImageLinkAdapter {
//                 image: Default::default(),
//                 name: value.name.clone().unwrap_or_default().into(),
//                 url: format!("dimple://genre/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             title: LinkAdapter {
//                 name: value.name.clone().unwrap_or_default().into(),
//                 url: format!("dimple://genre/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             sub_title: LinkAdapter {
//                 name: value.disambiguation.unwrap_or_default().into(),
//                 url: format!("dimple://genre/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//         }
//     }
// }

// impl From<Playlist> for CardAdapter {
//     fn from(value: Playlist) -> Self {
//         CardAdapter {
//             image: ImageLinkAdapter {
//                 image: Default::default(),
//                 name: value.name.clone().unwrap_or_default().into(),
//                 url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             title: LinkAdapter {
//                 name: value.name.clone().unwrap_or_default().into(),
//                 url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             ..Default::default()
//             // sub_title: LinkAdapter {
//             //     name: value.disambiguation.unwrap_or_default().into(),
//             //     url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
//             // },
//         }
//     }
// }

// impl From<Track> for CardAdapter {
//     fn from(value: Track) -> Self {
//         CardAdapter {
//             image: ImageLinkAdapter {
//                 image: Default::default(),
//                 name: value.title.clone().unwrap_or_default().into(),
//                 url: format!("dimple://track/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             title: LinkAdapter {
//                 name: value.title.clone().unwrap_or_default().into(),
//                 url: format!("dimple://track/{}", value.key.clone().unwrap_or_default()).into(),
//             },
//             ..Default::default()
//             // sub_title: LinkAdapter {
//             //     name: value.disambiguation.unwrap_or_default().into(),
//             //     url: format!("dimple://playlist/{}", value.key.clone().unwrap_or_default()).into(),
//             // },
//         }
//     }
// }
