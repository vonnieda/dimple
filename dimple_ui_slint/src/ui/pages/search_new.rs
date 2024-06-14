use dimple_core::model::Genre;
use dimple_core::model::Model;
use dimple_core::model::ReleaseGroup;
use slint::ComponentHandle;
use slint::ModelRc;
use slint::VecModel;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

pub fn search(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();

    std::thread::spawn(move || {
        log::info!("{}", url);
        let url = Url::parse(&url).unwrap();
        let query = url.path_segments().unwrap().next().unwrap();
        let query = percent_encoding::percent_decode_str(query).decode_utf8_lossy().to_string();

        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Navigator>().set_busy(true);
            let adapter = CardGridAdapter {
                cards: ModelRc::new(VecModel::<CardAdapter>::default()),
                ..Default::default()
            };
            ui.set_search(adapter);
            ui.set_page(Page::Search);
        }).unwrap();

        let results = librarian.search(&query).unwrap();
        for result in results {
            // TODO I'd like to have a simple way to throttle this, so that
            // like we queue up the incoming and only send sets of changes
            // every max like 200ms or something.
            let images = images.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ui.get_search();
                let cards: &slint::VecModel<CardAdapter> = slint::Model::as_any(&adapter.cards)
                    .downcast_ref().unwrap();
                let mut card: CardAdapter = model_card(&result);
                let index = slint::Model::row_count(cards);
                card.image.image = images.lazy_get(result, 200, 200, move |ui, image| {
                    let mut card = slint::Model::row_data(&ui.get_search().cards, index).unwrap();
                    card.image.image = image;
                    slint::Model::set_row_data(&ui.get_search().cards, index, card);
                });
                // TODO race condition with lazy_get, index might not be there yet.
                cards.push(card);
            }).unwrap();
        }

        ui.upgrade_in_event_loop(move |ui| {
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

fn release_group_card(release_group: &ReleaseGroup) -> CardAdapter {
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

fn genre_card(genre: &Genre) -> CardAdapter {
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

