use dimple_core::library::Library;
use dimple_core::model::{Artist, LibraryModel, Model, Playlist, Release};
use slint::{ComponentHandle as _, ModelRc};

use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::{AppWindow, CardAdapter, HomeAdapter, CardSectionAdapter, ImageLinkAdapter, LinkAdapter, Page};

pub fn home_init(app: &App) {
    let app1 = app.clone();
    // app.library.on_change(Box::new(|event| {
    //     if event.type_name == "Release" {

    //     }
    // }));
}

// For You (From your listens.)
// Recent Favorites
// New Releases
// Popular (With MusicBrainz listeners.)
// Old Favorites
pub fn home(app: &App) {    
    let app = app.clone();
    update_model(&app);
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::Home);
    }).unwrap();
}

fn update_model(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || { 
        let new_releases = app.library.query("
            SELECT * FROM Release ORDER BY date DESC LIMIT 10 
        ", ());

        let favorite_releases = app.library.query("
            SELECT Release.* 
            FROM Release 
            JOIN ArtistRef ON (ArtistRef.model_key = Release.key)
            JOIN Artist ON (Artist.key = ArtistRef.artist_key)
            JOIN 
                (SELECT artist,album,count(title) AS cnt 
                    FROM Event 
                    WHERE (event_type = 'track_played' OR event_type = 'track_restarted') 
                    GROUP BY artist,album) AS Ranks 
                ON (Release.title = Ranks.album AND Artist.name = Ranks.artist)
            ORDER BY Ranks.cnt DESC LIMIT 10;
        ", ());
        
        let favorite_artists = app.library.query("
            SELECT Artist.* 
            FROM Artist 
            JOIN 
                (SELECT artist,count(title) AS cnt 
                    FROM Event 
                    WHERE (event_type = 'track_played' OR event_type = 'track_restarted') 
                    GROUP BY artist) AS Ranks 
                ON (Artist.name = Ranks.artist)
            ORDER BY Ranks.cnt DESC LIMIT 10;
        ", ());
    
        let app = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let mut sections: Vec<CardSectionAdapter> = vec![];

            sections.push(CardSectionAdapter {
                title: "Newest Releases".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &new_releases, &app.library).as_slice().into(),
            });

            sections.push(CardSectionAdapter {
                title: "Favorite Releases".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &favorite_releases, &app.library).as_slice().into(),
            });

            sections.push(CardSectionAdapter {
                title: "Favorite Artists".into(),
                sub_title: Default::default(),
                cards: artist_cards(&app.images, &favorite_artists).as_slice().into(),
            });

            let adapter = ui.global::<HomeAdapter>();
            adapter.set_sections(sections.as_slice().into());
        }).unwrap();
    });
}

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(release.clone(), 200, 200, move |ui, image| {
                let adapter = ui.global::<HomeAdapter>();
                // let mut card = adapter.get_releases().row_data(index).unwrap();
                // card.image.image = image;
                // adapter.get_releases().set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.key.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn artist_cards(images: &ImageMangler, artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let mut card: CardAdapter = artist_card(&artist);
            card.image.image = images.lazy_get(artist.clone(), 200, 200, move |ui, image| {
                // let mut card = ui.get_artist_list().cards.row_data(index).unwrap();
                // card.image.image = image;
                // ui.get_artist_list().cards.set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.key.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

