use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Playlist;
use dimple_core::model::Model;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use dimple_core::db::Db;
use crate::ui::LinkAdapter;
use crate::ui::CardAdapter;
use crate::ui::GenreDetailsAdapter;

pub fn genre_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let genre: Genre = librarian.get(&Genre {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        let mut artists: Vec<Artist> = librarian
            .list(&Artist::default().into(), &Some(genre.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        artists.sort_by_key(|f| f.name.to_owned());
        // artists.reverse();

        let mut release_groups: Vec<ReleaseGroup> = librarian
            .list(&ReleaseGroup::default().into(), &Some(genre.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        release_groups.sort_by_key(|f| f.title.to_owned());
        // release_groups.reverse();

        let mut tracks: Vec<Track> = librarian
            .list(&Track::default().into(), &Some(genre.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        tracks.sort_by_key(|f| f.title.to_owned());
        // artists.reverse();

        let mut playlists: Vec<Playlist> = librarian
            .list(&Playlist::default().model(), &Some(genre.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        playlists.sort_by_key(|f| f.name.to_owned());
        // release_groups.reverse();

        // let related_genres: Vec<Genre> = librarian
        //     .list(&Genre::default().into(), Some(&Model::Artist(artist.clone())))
        //     .unwrap()
        //     .map(Into::into)
        //     .collect();

        ui.upgrade_in_event_loop(move |ui| {
            let artists: Vec<CardAdapter> = artists.iter().cloned()
                .enumerate()
                .map(|(index, artist)| {
                    let mut card: CardAdapter = artist.clone().into();
                    card.image.image = images.lazy_get(artist.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_genre_details().artists.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_genre_details().artists.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            let release_groups: Vec<CardAdapter> = release_groups.iter().cloned()
                .enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_genre_details().release_groups.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_genre_details().release_groups.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            let playlists: Vec<CardAdapter> = playlists.iter().cloned()
                .enumerate()
                .map(|(index, playlist)| {
                    let mut card: CardAdapter = playlist.clone().into();
                    card.image.image = images.lazy_get(playlist.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_genre_details().playlists.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_genre_details().playlists.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            let tracks: Vec<CardAdapter> = tracks.iter().cloned()
                .enumerate()
                .map(|(index, track)| {
                    let mut card: CardAdapter = track.clone().into();
                    card.image.image = images.lazy_get(track.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_genre_details().tracks.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_genre_details().tracks.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            // let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
            //     LinkAdapter {
            //         name: genre.name.unwrap().into(),
            //         url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
            //     }
            // }).collect();

            let links: Vec<LinkAdapter> = genre.links.iter().map(|link| {
                LinkAdapter {
                    name: link.into(),
                    url: link.into(),
                }
            }).collect();

            let mut adapter = GenreDetailsAdapter {
                card: genre.clone().into(),
                disambiguation: genre.disambiguation.clone().unwrap_or_default().into(),
                summary: genre.summary.clone().unwrap_or_default().into(),
                artists: ModelRc::from(artists.as_slice()),
                release_groups: ModelRc::from(release_groups.as_slice()),
                playlists: ModelRc::from(playlists.as_slice()),
                tracks: ModelRc::from(tracks.as_slice()),
                // related_genres: ModelRc::from(related_genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                dump: serde_json::to_string_pretty(&genre).unwrap().into(),
                ..Default::default()
            };
            adapter.card.image.image = images.get(genre.model(), 275, 275);
            ui.set_genre_details(adapter);
            ui.set_page(Page::GenreDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

