use dimple_core::model::Artist;
use dimple_librarian::librarian::Librarian;
use slint::ModelRc;
use crate::ui::AppWindow;
use crate::ui::Page;
use dimple_core::db::Db;
use crate::ui::CardGridAdapter;
use crate::ui::CardAdapter;
use crate::ui::LinkAdapter;
use crate::ui::ImageLinkAdapter;

// https://releases.slint.dev/1.5.1/docs/rust/slint/struct.image#sending-image-to-a-thread
pub fn artist_list(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let librarian = librarian.clone();
    std::thread::spawn(move || {
        let mut artists: Vec<_> = librarian.list(
            &Artist::default().into(), 
            None)
            .unwrap()
            .map(Into::<Artist>::into)
            .collect();
        artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());
        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = artists.iter().map(Into::into).collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
                ..Default::default()
            };
            ui.set_artist_list(adapter);
            ui.set_page(Page::ArtistList);
        }).unwrap();
    });
}

impl From<&Artist> for CardAdapter {
    fn from(value: &Artist) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(), // HOW NOW?!
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