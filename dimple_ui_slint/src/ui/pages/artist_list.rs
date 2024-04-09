use dimple_core::model::Artist;
use dimple_librarian::librarian::Librarian;
use slint::ModelRc;
use slint::VecModel;
use crate::ui::AppWindow;
use crate::ui::Page;
use dimple_core::db::Db;
use crate::ui::CardGridAdapter;
use crate::ui::CardAdapter;
use crate::ui::LinkAdapter;

pub fn artist_list(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let librarian = librarian.clone();
    std::thread::spawn(move || {
        let artists: Vec<_> = librarian.list(
            &Artist::default().into(), 
            None)
            .unwrap()
            .map(Into::<Artist>::into)
            .map(Into::<LinkAdapter>::into)
            .collect();
        // artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());
        // TODO should I be defining an Artist ViewModel in the View? And in here
        // I'm mapping between Artist and ArtistViewModel? Then where does image
        // loading and generalization of cards happen?
    //     let cards = artist_cards(artists, &librarian,
    //         Self::THUMBNAIL_WIDTH, 
    //         Self::THUMBNAIL_WIDTH);
        ui.upgrade_in_event_loop(move |ui| {
            artists.as_ptr();
            let adapter = CardGridAdapter {
                // cards: ModelRc::from(artists.as_slice()),
                ..Default::default()
            };
            ui.set_artist_list(adapter);
            ui.set_page(Page::ArtistList);
        }).unwrap();
    });
}

// fn artist_card(artist: &Artist, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Artist(artist.clone()), width, height),
//             link: Link {
//                 name: artist.name.clone().unwrap_or_default(),
//                 url: format!("dimple://artist/{}", artist.key.str()),
//             },
//         },
//         title: Link {
//             name: artist.name.clone().unwrap_or_default(),
//             url: format!("dimple://artist/{}", artist.key.str()),
//         },
//         sub_title: Link {
//             name: artist.disambiguation.clone().unwrap_or_default(),
//             url: format!("dimple://artist/{}", artist.key.str()),
//         },
//     }
// }

impl From<Artist> for LinkAdapter {
    fn from(value: Artist) -> Self {
        todo!()
    }
}

impl From<Artist> for CardAdapter {
    fn from(value: Artist) -> Self {
        CardAdapter {
            title: LinkAdapter {
                name: value.name.unwrap_or("Unknown Artist".to_string()).into(),
                url: Default::default(),
            },
            ..Default::default()
        }
    }
}