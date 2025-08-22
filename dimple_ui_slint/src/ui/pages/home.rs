use dimple_core::library::Library;
use dimple_core::model::{Artist, Release};
use slint::ComponentHandle as _;

use crate::ui::app_window_controller::App;
use crate::ui::{CardAdapter, HomeAdapter, CardSectionAdapter, ImageLinkAdapter, LinkAdapter, Page};

pub fn home_init(app: &App) {
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
        let newest_releases = app.library.query("
            SELECT * FROM Release ORDER BY date DESC LIMIT 10 
        ", ());

        let favorite_releases = app.library.query("
            SELECT Release.* 
            FROM Release 
            JOIN ArtistRef ON (ArtistRef.model_id = Release.id)
            JOIN Artist ON (Artist.id = ArtistRef.artist_id)
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
                url: "dimple://home/newest-releases".to_string().into(),
                cards: release_cards(&newest_releases, &app.library).as_slice().into(),
            });

            sections.push(CardSectionAdapter {
                title: "Favorite Releases".into(),
                sub_title: Default::default(),
                url: "dimple://home/favorite-releases".to_string().into(),
                cards: release_cards(&favorite_releases, &app.library).as_slice().into(),
            });

            sections.push(CardSectionAdapter {
                title: "Favorite Artists".into(),
                sub_title: Default::default(),
                url: "dimple://home/favorite-artists".to_string().into(),
                cards: artist_cards(&favorite_artists).as_slice().into(),
            });

            let adapter = ui.global::<HomeAdapter>();
            adapter.set_sections(sections.as_slice().into());
        }).unwrap();
    });
}

fn release_cards(releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.id.clone().unwrap_or_default().into(),
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn artist_cards(artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let card: CardAdapter = artist_card(&artist);
            card
        })
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.id.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

