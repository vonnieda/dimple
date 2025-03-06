use dimple_core::library::Library;
use dimple_core::model::{Artist, LibraryModel, Model, Playlist, Release};
use slint::{ComponentHandle as _, ModelRc};

use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::{AppWindow, CardAdapter, HomeAdapter, HomeSectionAdapter, ImageLinkAdapter, LinkAdapter, Page};

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
        let new_releases = new_releases(&app.library);
        let made_for_you = made_for_you(&app.library);
        let jump_back_in = jump_back_in(&app.library);
        let most_played = most_played(&app.library);

        let app = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let mut sections: Vec<HomeSectionAdapter> = vec![];

            sections.push(HomeSectionAdapter {
                title: "New Releases".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &new_releases, &app.library).as_slice().into(),
            });

            sections.push(HomeSectionAdapter {
                title: "Made For You".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &made_for_you, &app.library).as_slice().into(),
            });

            sections.push(HomeSectionAdapter {
                title: "Jump Back In".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &jump_back_in, &app.library).as_slice().into(),
            });

            sections.push(HomeSectionAdapter {
                title: "Most Played".into(),
                sub_title: Default::default(),
                cards: release_cards(&app.images, &most_played, &app.library).as_slice().into(),
            });

            let adapter = ui.global::<HomeAdapter>();
            adapter.set_sections(sections.as_slice().into());
        }).unwrap();
    });
}

// New releases, filtered by things you like.
fn new_releases(library: &Library) -> Vec<Release> {
    library.query("
        SELECT * FROM Release ORDER BY date DESC LIMIT 10 
    ", ())
}

// Daily, mood, genre playlists. Will generate on demand I think. For now/
fn made_for_you(library: &Library) -> Vec<Release> {
    library.query("
        SELECT * FROM Release ORDER BY random() LIMIT 10 
    ", ())
}

// Things you listened to recently but didn't finish.
fn jump_back_in(library: &Library) -> Vec<Release> {
    library.query("
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
        WHERE cnt > 3
        ORDER BY random() LIMIT 10;
        ", ())
}

// Random (daily?) assortment of most played items.
fn most_played(library: &Library) -> Vec<Release> {
    library.query("
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
    ", ())
}


// select row_number() over (order by count(artist) desc),count(artist),artist from event group by artist limit 10;

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
