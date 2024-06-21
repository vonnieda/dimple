use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::ReleaseGroup;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::ArtistDetailsAdapter;
use crate::ui::CardAdapter;
use crate::ui::LinkAdapter;

pub fn artist_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();

    // Turn on the spinner, set an empty model, and go to the page.
    ui.upgrade_in_event_loop(|ui| {
        ui.global::<Navigator>().set_busy(true);

        let adapter = ArtistDetailsAdapter {
            ..Default::default()
        };

        ui.set_artist_details(adapter);
        ui.set_page(Page::ArtistDetails);
    }).unwrap();

    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        // Load the artist by key.
        let artist: Artist = librarian.get(&Artist {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        // Set the available artist properties on the UI, and start loading the
        // artist image.
        {
            let images = images.clone();
            let artist = artist.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let links: Vec<LinkAdapter> = artist.links.iter().map(|link| {
                    LinkAdapter {
                        name: link.into(),
                        url: link.into(),
                    }
                }).collect();
    
                let mut adapter = ui.get_artist_details();
                adapter.card.image.image = images.lazy_get(artist.model(), 275, 275, |ui, image| {
                    let mut model = ui.get_artist_details();
                    model.card.image.image = image;
                    ui.set_artist_details(model);
                });
                let card: CardAdapter = artist.clone().into();
                adapter.card.title = card.title;
                adapter.card.sub_title = card.sub_title;
                adapter.disambiguation = artist.disambiguation.clone().unwrap_or_default().into();
                adapter.summary = artist.summary.clone().unwrap_or_default().into();
                adapter.links = ModelRc::from(links.as_slice());
                adapter.dump = serde_json::to_string_pretty(&artist).unwrap().into();    
                ui.set_artist_details(adapter);
            }).unwrap();
        }
    
        let mut genres: Vec<Genre> = librarian
            .list(&Genre::default().into(), &Some(artist.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        genres.sort_by_key(|genre| genre.name.clone().unwrap_or_default().to_lowercase());

        let mut release_groups: Vec<_> = librarian
            .list(&ReleaseGroup::default().into(), &Some(artist.model()))
            .unwrap()
            .map(ReleaseGroup::from)
            .collect();
        release_groups.sort_by_key(|r| r.first_release_date.to_owned());
        release_groups.reverse();

        ui.upgrade_in_event_loop(move |ui| {
            let albums: Vec<CardAdapter> = release_groups.iter().cloned()
                .enumerate()
                .map(|(index, release)| {
                    let mut card: CardAdapter = release.clone().into();
                    card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_artist_details().albums.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_artist_details().albums.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
                LinkAdapter {
                    name: genre.name.unwrap().into(),
                    url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
                }
            }).collect();

            let mut adapter = ui.get_artist_details();
            adapter.albums = ModelRc::from(albums.as_slice());
            adapter.genres = ModelRc::from(genres.as_slice());
            ui.set_artist_details(adapter);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

